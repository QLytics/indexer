use crate::schema::*;
use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::views::BlockView;

#[derive(Identifiable, Insertable, Queryable)]
#[diesel(primary_key(hash))]
pub struct Block {
    pub hash: String,
    pub height: String,
    pub prev_hash: String,
    pub timestamp: NaiveDateTime,
    pub total_supply: String,
    pub gas_price: String,
    pub author_account_id: String,
}

impl Block {
    pub fn new(block_view: &BlockView, timestamp: NaiveDateTime) -> Self {
        Self {
            hash: block_view.header.hash.to_string(),
            height: block_view.header.height.to_string(),
            prev_hash: block_view.header.prev_hash.to_string(),
            timestamp,
            total_supply: block_view.header.total_supply.to_string(),
            gas_price: block_view.header.gas_price.to_string(),
            author_account_id: block_view.author.to_string(),
        }
    }
}
