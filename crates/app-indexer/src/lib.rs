mod error;

use actix_web::web;
use futures::StreamExt;
use near_lake_framework::{
    near_indexer_primitives::{IndexerTransactionWithOutcome, StreamerMessage},
    LakeConfigBuilder,
};
use near_ql_db::Database;
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

pub fn start_indexing(db: web::Data<Database>) {
    actix_web::rt::spawn(async move {
        let config = LakeConfigBuilder::default()
            .mainnet()
            .start_block_height(60000000)
            .build()
            .expect("Failed to build LakeConfig");

        let (sender, stream) = near_lake_framework::streamer(config);

        let time = Arc::new(RwLock::new(Instant::now()));
        let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
            .map(|msg| handle_streamer_message(db.clone(), msg, time.clone()))
            .buffer_unordered(1usize);

        while let Some(_handle_message) = handlers.next().await {}
        drop(handlers);

        sender.await.unwrap().unwrap();
    });
}

async fn handle_streamer_message(
    db: web::Data<Database>,
    msg: StreamerMessage,
    time: Arc<RwLock<Instant>>,
) {
    if time.read().unwrap().elapsed() > Duration::from_secs(10) {
        println!("Block height: {}", msg.block.header.height);
        *time.write().unwrap() = Instant::now();
    }

    let block_hash = msg.block.header.hash;
    let timestamp = msg.block.header.timestamp_nanosec;
    for shard in msg.shards {
        let chunk = if let Some(chunk) = shard.chunk {
            chunk
        } else {
            continue;
        };
        let chunk_hash = chunk.header.chunk_hash;
        for (chunk_index, IndexerTransactionWithOutcome { transaction, .. }) in
            chunk.transactions.into_iter().enumerate()
        {
            db.insert_transaction(
                transaction,
                block_hash,
                chunk_hash,
                chunk_index as i32,
                timestamp.into(),
            )
            .unwrap();
        }
    }
}
