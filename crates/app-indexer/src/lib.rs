mod error;

pub use error::Error;

use chrono::NaiveDateTime;
use futures::StreamExt;
use near_lake_framework::{
    near_indexer_primitives::{IndexerTransactionWithOutcome, StreamerMessage},
    LakeConfigBuilder,
};
use near_ql_db::{DbConn, ExecutionOutcome, Receipt, Transaction, TransactionAction};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_stream::wrappers::ReceiverStream;

pub async fn start_indexing(db: DbConn) -> Result<(), Error> {
    let config = LakeConfigBuilder::default()
        .mainnet()
        .start_block_height(0)
        .start_block_height(
            env::var("START_BLOCK_HEIGHT")
                .map(|s| s.parse::<u64>().unwrap_or_default())
                .unwrap_or_default(),
        )
        .build()
        // TODO: LakeConfigBuildError
        .unwrap();

    let (sender, stream) = near_lake_framework::streamer(config);

    let time = Arc::new(RwLock::new(Instant::now()));
    let transactions = Arc::new(RwLock::new(vec![]));
    let transaction_actions = Arc::new(RwLock::new(vec![]));
    let receipts = Arc::new(RwLock::new(vec![]));
    let execution_outcomes = Arc::new(RwLock::new(vec![]));
    let mut handlers = ReceiverStream::new(stream)
        .map(|msg| {
            handle_streamer_message(
                msg,
                time.clone(),
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

async fn handle_streamer_message(
    msg: StreamerMessage,
    time: Arc<RwLock<Instant>>,
    transactions: Arc<RwLock<Vec<Transaction>>>,
    transaction_actions: Arc<RwLock<Vec<TransactionAction>>>,
    receipts: Arc<RwLock<Vec<Receipt>>>,
    execution_outcomes: Arc<RwLock<Vec<ExecutionOutcome>>>,
) {
    if time.read().elapsed() > Duration::from_secs(10) {
        println!("Block height: {}", msg.block.header.height);
        *time.write() = Instant::now();
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
}
