mod error;

use actix_web::web;
use futures::StreamExt;
use near_lake_framework::{
    near_indexer_primitives::{IndexerTransactionWithOutcome, StreamerMessage},
    LakeConfigBuilder,
};
use near_ql_db::Database;

pub fn start_indexing(database: web::Data<Database>) {
    actix_web::rt::spawn(async move {
        let config = LakeConfigBuilder::default()
            .mainnet()
            .start_block_height(60000000)
            .build()
            .expect("Failed to build LakeConfig");

        let (sender, stream) = near_lake_framework::streamer(config);

        let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
            .map(handle_streamer_message)
            .buffer_unordered(1usize);

        while let Some(_handle_message) = handlers.next().await {}
        drop(handlers);

        sender.await.unwrap().unwrap();
    });
}

async fn handle_streamer_message(msg: StreamerMessage) {
    for shard in msg.shards {
        let chunk = if let Some(chunk) = shard.chunk {
            chunk
        } else {
            continue;
        };
        for IndexerTransactionWithOutcome { transaction, .. } in chunk.transactions {
            if transaction.receiver_id == "pixeltoken.near".to_string().try_into().unwrap() {
                dbg!(transaction);
            }
        }
    }
}
