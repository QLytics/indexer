use crate::schema::*;
use near_lake_framework::near_indexer_primitives::{CryptoHash, IndexerChunkView};

#[derive(Identifiable, Insertable, Queryable)]
#[diesel(primary_key(chunk_hash))]
pub struct Chunk {
    pub chunk_hash: String,
    pub block_hash: String,
    pub shard_id: String,
    pub signature: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub author_account_id: String,
}

impl Chunk {
    pub fn new(chunk_view: &IndexerChunkView, block_hash: CryptoHash) -> Self {
        Self {
            chunk_hash: chunk_view.header.chunk_hash.to_string(),
            block_hash: block_hash.to_string(),
            shard_id: chunk_view.header.shard_id.to_string(),
            signature: chunk_view.header.signature.to_string(),
            gas_limit: chunk_view.header.gas_limit.to_string(),
            gas_used: chunk_view.header.gas_used.to_string(),
            author_account_id: chunk_view.author.to_string(),
        }
    }
}
