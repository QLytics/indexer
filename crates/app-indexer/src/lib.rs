#![feature(drain_filter)]

mod error;

pub use error::Error;

use chrono::{DateTime, NaiveDateTime, Utc};
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_lake_framework::{
    near_indexer_primitives::{
        views::ReceiptEnumView, CryptoHash, IndexerTransactionWithOutcome, StreamerMessage,
    },
    LakeConfigBuilder,
};
use near_ql_db::{DbConn, ExecutionOutcome, Receipt, Transaction, TransactionAction};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    env,
    sync::Arc,
    time::{Duration, Instant},
};

pub async fn start_indexing(db: DbConn) -> Result<(), Error> {
    let config = LakeConfigBuilder::default()
        .mainnet()
        .start_block_height(
            env::var("START_BLOCK_HEIGHT")
                .map(|s| s.parse::<u64>().unwrap_or_default())
                .unwrap_or_default(),
        )
        .build()
        // TODO: LakeConfigBuildError
        .unwrap();
    let client = Arc::new(JsonRpcClient::connect("https://rpc.mainnet.near.org"));

    let (sender, mut stream) = near_lake_framework::streamer(config);

    let time = Arc::new(RwLock::new(Instant::now()));
    let eta = Arc::new(RwLock::new(VecDeque::new()));
    let receipt_id_to_tx_hash = Arc::new(RwLock::new(HashMap::new()));
    let data_id_to_tx_hash = Arc::new(RwLock::new(HashMap::new()));

    let transactions = Arc::new(RwLock::new(vec![]));
    let transaction_actions = Arc::new(RwLock::new(vec![]));
    let receipts = Arc::new(RwLock::new(vec![]));
    let execution_outcomes = Arc::new(RwLock::new(vec![]));

    let misses = Arc::new(RwLock::new(0));

    while let Some(msg) = stream.recv().await {
        handle_streamer_message(
            client.clone(),
            msg,
            time.clone(),
            eta.clone(),
            receipt_id_to_tx_hash.clone(),
            data_id_to_tx_hash.clone(),
            transactions.clone(),
            transaction_actions.clone(),
            receipts.clone(),
            execution_outcomes.clone(),
            misses.clone(),
        )
        .await?;

        {
            let mut transactions = transactions.write();
            db.write().insert_transactions(&*transactions).unwrap();
            *transactions = vec![];
        }

        {
            let mut transaction_actions = transaction_actions.write();
            db.write()
                .insert_transaction_actions(&*transaction_actions)
                .unwrap();
            *transaction_actions = vec![];
        }

        {
            let mut receipts = receipts.write();
            db.write().insert_receipts(&*receipts).unwrap();
            *receipts = vec![];
        }

        {
            let mut execution_outcomes = execution_outcomes.write();
            db.write()
                .insert_execution_outcomes(&*execution_outcomes)
                .unwrap();
            *execution_outcomes = vec![];
        }
    }

    sender.await.unwrap().unwrap();
    Ok(())
}

#[allow(clippy::await_holding_lock)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
async fn handle_streamer_message(
    client: Arc<JsonRpcClient>,
    msg: StreamerMessage,
    time: Arc<RwLock<Instant>>,
    eta: Arc<RwLock<VecDeque<(Duration, u64)>>>,
    receipt_id_to_tx_hash: Arc<RwLock<HashMap<CryptoHash, CryptoHash>>>,
    data_id_to_tx_hash: Arc<RwLock<HashMap<CryptoHash, CryptoHash>>>,
    transactions: Arc<RwLock<Vec<Transaction>>>,
    transaction_actions: Arc<RwLock<Vec<TransactionAction>>>,
    receipts: Arc<RwLock<Vec<Receipt>>>,
    execution_outcomes: Arc<RwLock<Vec<ExecutionOutcome>>>,
    misses: Arc<RwLock<u32>>,
) -> Result<(), Error> {
    {
        let mut time = time.write();
        let elapsed = time.elapsed();
        if elapsed > Duration::from_secs(10) {
            *time = Instant::now();
            drop(time);
            let current_block_height = get_current_block_height(&client).await?;
            let mut eta = eta.write();
            eta.push_back((elapsed, msg.block.header.height));
            if eta.len() > 1 {
                if eta.len() > 5 {
                    eta.pop_front();
                }

                let (_, first_block_height) = eta.front().unwrap();
                let (_, last_block_height) = eta.back().unwrap();
                let total_dur: Duration = eta.iter().map(|(t, _)| t).sum();
                let total_blocks = last_block_height - first_block_height;
                let blocks_per_millis = total_blocks as f64 / total_dur.as_millis() as f64;
                let utc: DateTime<Utc> = Utc::now();

                let eta =
                    (current_block_height - msg.block.header.height) as f64 / blocks_per_millis;
                let eta = Duration::from_millis(eta as u64);

                println!(
                    "[{}] Height: {}, BPS: {:.1}, Misses: {}, ETA: {}",
                    utc.format("%Y-%m-%d %H:%M:%S"),
                    msg.block.header.height,
                    blocks_per_millis as f32 * 1_000.,
                    misses.read(),
                    humantime::Duration::from(eta)
                );
            }
        }
    }

    let block_hash = msg.block.header.hash;
    let timestamp = msg.block.header.timestamp_nanosec as i64 / 1_000_000;
    let timestamp = NaiveDateTime::from_timestamp(timestamp / 1_000, timestamp as u32 % 1_000);

    msg.shards
        .par_iter()
        .filter_map(|shard| {
            shard.receipt_execution_outcomes.iter().for_each(|outcome| {
                outcome
                    .execution_outcome
                    .outcome
                    .receipt_ids
                    .iter()
                    .for_each(|receipt_id| {
                        let mut receipt_id_to_tx_hash = receipt_id_to_tx_hash.write();
                        if let Some(hash) = receipt_id_to_tx_hash
                            .get(&outcome.execution_outcome.id)
                            .cloned()
                        {
                            receipt_id_to_tx_hash.insert(*receipt_id, hash);
                        } else {
                            // eprintln!("Could not find parent for receipt {}", receipt_id);
                            // TODO strict mode
                        }
                    });
            });
            shard.chunk.as_ref()
        })
        .for_each(|chunk| {
            chunk.transactions.par_iter().for_each(|transaction| {
                if let Some(receipt) = &transaction.outcome.receipt {
                    receipt_id_to_tx_hash
                        .write()
                        .insert(receipt.receipt_id, transaction.transaction.hash);
                }
                transaction
                    .outcome
                    .execution_outcome
                    .outcome
                    .receipt_ids
                    .iter()
                    .for_each(|receipt_id| {
                        receipt_id_to_tx_hash
                            .write()
                            .insert(*receipt_id, transaction.transaction.hash);
                    });
            });
            chunk.receipts.iter().for_each(|receipt| {
                if let ReceiptEnumView::Data { data_id, .. } = receipt.receipt {
                    let removed_data_id = data_id_to_tx_hash.write().remove(&data_id);
                    if let Some(tx_hash) = removed_data_id {
                        receipt_id_to_tx_hash
                            .write()
                            .insert(receipt.receipt_id, tx_hash);
                    } else {
                        // TODO strict mode
                    }
                }
            });
        });

    msg.shards.par_iter().for_each(|shard| {
        let chunk = if let Some(chunk) = &shard.chunk {
            chunk
        } else {
            return;
        };
        let chunk_hash = chunk.header.chunk_hash;

        chunk
            .receipts
            .par_iter()
            .enumerate()
            .for_each(|(chunk_index, receipt_view)| {
                if let Some(outcome) = shard
                    .receipt_execution_outcomes
                    .iter()
                    .find(|r| r.execution_outcome.id == receipt_view.receipt_id)
                {
                    let execution_outcome = ExecutionOutcome::new(
                        receipt_view,
                        block_hash,
                        chunk_index as i32,
                        timestamp,
                        &outcome.execution_outcome.outcome,
                        shard.shard_id,
                    );
                    execution_outcomes.write().push(execution_outcome);
                }

                let removed_receipt = receipt_id_to_tx_hash
                    .write()
                    .get(&receipt_view.receipt_id)
                    .cloned();
                if let Some(tx_hash) = removed_receipt {
                    let receipt = Receipt::new(
                        receipt_view,
                        block_hash,
                        chunk_hash,
                        chunk_index as i32,
                        timestamp,
                        tx_hash,
                    );
                    receipts.write().push(receipt);

                    if let ReceiptEnumView::Action {
                        output_data_receivers,
                        ..
                    } = &receipt_view.receipt
                    {
                        output_data_receivers.iter().for_each(|receiver| {
                            data_id_to_tx_hash.write().insert(receiver.data_id, tx_hash);
                        });
                    }
                } else {
                    let mut misses = misses.write();
                    *misses += 1;
                }
            });

        chunk.transactions.par_iter().enumerate().for_each(
            |(
                chunk_index,
                IndexerTransactionWithOutcome {
                    transaction,
                    outcome,
                },
            )| {
                transaction.actions.par_iter().enumerate().for_each(
                    |(transaction_index, action_view)| {
                        let transaction_action = TransactionAction::new(
                            transaction,
                            transaction_index as i32,
                            action_view,
                        );
                        transaction_actions.write().push(transaction_action);
                    },
                );

                let transaction = Transaction::new(
                    transaction,
                    block_hash,
                    chunk_hash,
                    chunk_index as i32,
                    timestamp,
                    &outcome.execution_outcome.outcome,
                );
                transactions.write().push(transaction);
            },
        );

        let mut outcome_receipts: Vec<_> = msg
            .shards
            .iter()
            .flat_map(|shard| {
                shard.receipt_execution_outcomes.iter().flat_map(|outcome| {
                    outcome
                        .execution_outcome
                        .outcome
                        .receipt_ids
                        .iter()
                        .map(|receipt_id| (outcome.execution_outcome.id, *receipt_id))
                })
            })
            .collect();
        let mut retries_left = 5;
        loop {
            outcome_receipts.drain_filter(|(executed_receipt_id, produced_receipt_id)| {
                let mut receipt_id_to_tx_hash = receipt_id_to_tx_hash.write();
                if let Some(tx_hash) = receipt_id_to_tx_hash.get(executed_receipt_id).cloned() {
                    receipt_id_to_tx_hash.insert(*produced_receipt_id, tx_hash);
                    true
                } else {
                    false
                }
            });
            retries_left -= 1;
            if retries_left == 0 || outcome_receipts.is_empty() {
                break;
            }
        }
    });

    Ok(())
}

async fn get_current_block_height(client: &Arc<JsonRpcClient>) -> Result<u64, Error> {
    let status = client.call(methods::status::RpcStatusRequest).await?;

    Ok(status.sync_info.latest_block_height)
}
