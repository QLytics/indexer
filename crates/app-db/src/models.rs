use crate::schema::*;
use bigdecimal::BigDecimal;

#[derive(AsChangeset, Insertable)]
#[table_name = "transactions"]
pub struct Transaction {
    pub hash: String,
    pub block_hash: String,
    pub chunk_hash: String,
    pub chunk_index: i32,
    pub timestamp: BigDecimal,
    pub signer_id: String,
    pub public_key: String,
    pub nonce: BigDecimal,
    pub receiver_id: String,
    pub signature: String,
}
