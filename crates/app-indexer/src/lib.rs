mod error;

pub use error::Error;

use chrono::NaiveDateTime;
use futures::StreamExt;
use near_lake_framework::{
    near_indexer_primitives::{IndexerTransactionWithOutcome, StreamerMessage},
    LakeConfigBuilder,
};
use near_ql_db::{DbConn, Receipt, Transaction, TransactionAction};
use rayon::prelude::*;
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tokio_stream::wrappers::ReceiverStream;

pub async fn start_indexing(db: DbConn) -> Result<(), Error> {
    let config = LakeConfigBuilder::default()
        .mainnet()
        .start_block_height(60000000)
        .build()
        // TODO: LakeConfigBuildError
        .unwrap();

    let (sender, stream) = near_lake_framework::streamer(config);

    let time = Arc::new(RwLock::new(Instant::now()));
    let mut handlers = ReceiverStream::new(stream)
        .map(|msg| handle_streamer_message(db.clone(), msg, time.clone()))
        .buffer_unordered(1usize);

    while let Some(_handle_message) = handlers.next().await {}
    drop(handlers);

    sender.await.unwrap().unwrap();
    Ok(())
}

async fn handle_streamer_message(db: DbConn, msg: StreamerMessage, time: Arc<RwLock<Instant>>) {
    if time.read().unwrap().elapsed() > Duration::from_secs(10) {
        println!("Block height: {}", msg.block.header.height);
        *time.write().unwrap() = Instant::now();
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
                        let receipt = Receipt::new(
                            receipt,
                            block_hash,
                            chunk_hash,
                            chunk_index as i32,
                            timestamp,
                        );
                        db.write().insert_receipt(receipt).unwrap();
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
                        transaction.actions.iter().enumerate().for_each(
                            |(transaction_index, action_view)| {
                                let transaction_action = TransactionAction::new(
                                    &transaction,
                                    transaction_index as i32,
                                    action_view,
                                );
                                db.write()
                                    .insert_transaction_action(transaction_action)
                                    .unwrap();
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
                        db.write().insert_transaction(transaction).unwrap();
                    },
                )
            },
        );
    });
}
