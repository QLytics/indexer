use chrono::NaiveDateTime;
use graphql_client::GraphQLQuery;
use near_lake_framework::near_indexer_primitives::{
    views::BlockView, CryptoHash, IndexerChunkView,
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/block-query.graphql",
    response_derives = "Debug"
)]
pub struct AddBlocks;

impl add_blocks::NewBlock {
    pub fn new(block_view: &BlockView, timestamp: NaiveDateTime) -> Self {
        Self {
            hash: block_view.header.hash.to_string(),
            height: block_view.header.height.to_string(),
            prev_hash: block_view.header.prev_hash.to_string(),
            timestamp: timestamp.timestamp().to_string(),
            total_supply: block_view.header.total_supply.to_string(),
            gas_price: block_view.header.gas_price.to_string(),
            author_account_id: block_view.author.to_string(),
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/chunk-query.graphql",
    response_derives = "Debug"
)]
pub struct AddChunks;

impl add_chunks::NewChunk {
    pub fn new(chunk_view: &IndexerChunkView, block_hash: CryptoHash) -> Self {
        Self {
            hash: chunk_view.header.chunk_hash.to_string(),
            block_hash: block_hash.to_string(),
            shard_id: chunk_view.header.shard_id.to_string(),
            signature: chunk_view.header.signature.to_string(),
            gas_limit: chunk_view.header.gas_limit.to_string(),
            gas_used: chunk_view.header.gas_used.to_string(),
            author_account_id: chunk_view.author.to_string(),
        }
    }
}

pub use {add_blocks::NewBlock, add_chunks::NewChunk};
