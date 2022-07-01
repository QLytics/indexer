#![feature(drain_filter)]

mod error;
mod log;
mod receipt;
mod transaction;

pub use error::Error;

use chrono::NaiveDateTime;
use near_jsonrpc_client::JsonRpcClient;
use near_lake_framework::{
    near_indexer_primitives::{views::ReceiptEnumView, CryptoHash, StreamerMessage},
    LakeConfigBuilder,
};
use near_ql_db::{
    DataReceipt, DbConn, ExecutionOutcome, ExecutionOutcomeReceipt, Receipt, Transaction,
    TransactionAction,
};
use parking_lot::RwLock;
use rayon::prelude::*;
use receipt::{handle_chunk_receipts, handle_shard_receipts};
use std::{
    collections::{HashMap, VecDeque},
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use transaction::handle_transactions;

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
    let data_receipts = Arc::new(RwLock::new(vec![]));
    let execution_outcomes = Arc::new(RwLock::new(vec![]));
    let execution_outcome_receipts = Arc::new(RwLock::new(vec![]));

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
            data_receipts.clone(),
            execution_outcomes.clone(),
            execution_outcome_receipts.clone(),
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
            let mut data_receipts = data_receipts.write();
            db.write().insert_data_receipts(&*data_receipts).unwrap();
            *data_receipts = vec![];
        }

        {
            let mut execution_outcomes = execution_outcomes.write();
            db.write()
                .insert_execution_outcomes(&*execution_outcomes)
                .unwrap();
            *execution_outcomes = vec![];
        }

        {
            let mut execution_outcome_receipts = execution_outcome_receipts.write();
            db.write()
                .insert_execution_outcome_receipts(&*execution_outcome_receipts)
                .unwrap();
            *execution_outcome_receipts = vec![];
        }
    }

    sender.await.unwrap().unwrap();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
    data_receipts: Arc<RwLock<Vec<DataReceipt>>>,
    execution_outcomes: Arc<RwLock<Vec<ExecutionOutcome>>>,
    execution_outcome_receipts: Arc<RwLock<Vec<ExecutionOutcomeReceipt>>>,
    misses: Arc<RwLock<u32>>,
) -> Result<(), Error> {
    log::log(msg.block.header.height, &client, &time, &eta, &misses).await?;

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

        handle_chunk_receipts(
            shard,
            chunk,
            block_hash,
            chunk_hash,
            timestamp,
            &receipts,
            &data_receipts,
            &execution_outcomes,
            &execution_outcome_receipts,
            &receipt_id_to_tx_hash,
            &data_id_to_tx_hash,
            &misses,
        );

        handle_transactions(
            chunk,
            chunk_hash,
            block_hash,
            timestamp,
            &transactions,
            &transaction_actions,
        );
    });

    handle_shard_receipts(&msg, &receipt_id_to_tx_hash);

    Ok(())
}
