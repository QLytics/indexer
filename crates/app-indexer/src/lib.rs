mod error;

pub use error::Error;

use chrono::{DateTime, NaiveDateTime, Utc};
use futures::StreamExt;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_lake_framework::{
    near_indexer_primitives::{IndexerTransactionWithOutcome, StreamerMessage},
    LakeConfigBuilder,
};
use near_ql_db::{DbConn, ExecutionOutcome, Receipt, Transaction, TransactionAction};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::{
    collections::VecDeque,
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_stream::wrappers::ReceiverStream;

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

    let (sender, stream) = near_lake_framework::streamer(config);

    let time = Arc::new(RwLock::new(Instant::now()));
    let eta = Arc::new(RwLock::new(VecDeque::new()));
    let transactions = Arc::new(RwLock::new(vec![]));
    let transaction_actions = Arc::new(RwLock::new(vec![]));
    let receipts = Arc::new(RwLock::new(vec![]));
    let execution_outcomes = Arc::new(RwLock::new(vec![]));

    let mut handlers = ReceiverStream::new(stream)
        .map(|msg| {
            handle_streamer_message(
                client.clone(),
                msg,
                time.clone(),
                eta.clone(),
                transactions.clone(),
                transaction_actions.clone(),
                receipts.clone(),
                execution_outcomes.clone(),
            )
        })
        .buffered(num_cpus::get());

    while let Some(_handle_message) = handlers.next().await {
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
    drop(handlers);

    sender.await.unwrap().unwrap();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::await_holding_lock)]
async fn handle_streamer_message(
    client: Arc<JsonRpcClient>,
    msg: StreamerMessage,
    time: Arc<RwLock<Instant>>,
    eta: Arc<RwLock<VecDeque<(Duration, u64)>>>,
    transactions: Arc<RwLock<Vec<Transaction>>>,
    transaction_actions: Arc<RwLock<Vec<TransactionAction>>>,
    receipts: Arc<RwLock<Vec<Receipt>>>,
    execution_outcomes: Arc<RwLock<Vec<ExecutionOutcome>>>,
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
                    "[{}] Height: {}, BPS: {:.1}, ETA: {}",
                    utc.format("%Y-%m-%d %H:%M:%S"),
                    msg.block.header.height,
                    blocks_per_millis as f32 * 1_000.,
                    humantime::Duration::from(eta)
                );
            }
        }
    }

    let block_hash = msg.block.header.hash;
    let timestamp = msg.block.header.timestamp_nanosec as i64 / 1_000_000;
    let timestamp = NaiveDateTime::from_timestamp(timestamp / 1_000, timestamp as u32 % 1_000);

    msg.shards.into_par_iter().for_each(|shard| {
        let chunk = if let Some(chunk) = shard.chunk {
            chunk
        } else {
            return;
        };
        let chunk_hash = chunk.header.chunk_hash;

        rayon::join(
            || {
                chunk
                    .receipts
                    .into_par_iter()
                    .enumerate()
                    .for_each(|(chunk_index, receipt)| {
                        if let Some(outcome) = shard
                            .receipt_execution_outcomes
                            .iter()
                            .find(|r| r.execution_outcome.id == receipt.receipt_id)
                        {
                            let execution_outcome = ExecutionOutcome::new(
                                &receipt,
                                block_hash,
                                chunk_index as i32,
                                timestamp,
                                &outcome.execution_outcome.outcome,
                                shard.shard_id,
                            );
                            execution_outcomes.write().push(execution_outcome);
                        }

                        let receipt = Receipt::new(
                            &receipt,
                            block_hash,
                            chunk_hash,
                            chunk_index as i32,
                            timestamp,
                        );
                        receipts.write().push(receipt);
                    })
            },
            || {
                chunk.transactions.into_par_iter().enumerate().for_each(
                    |(
                        chunk_index,
                        IndexerTransactionWithOutcome {
                            transaction,
                            outcome,
                            ..
                        },
                    )| {
                        transaction.actions.par_iter().enumerate().for_each(
                            |(transaction_index, action_view)| {
                                let transaction_action = TransactionAction::new(
                                    &transaction,
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
                            outcome.execution_outcome.outcome,
                        );
                        transactions.write().push(transaction);
                    },
                );
            },
        );
    });

    Ok(())
}

async fn get_current_block_height(client: &Arc<JsonRpcClient>) -> Result<u64, Error> {
    let status = client.call(methods::status::RpcStatusRequest).await?;

    Ok(status.sync_info.latest_block_height)
}
