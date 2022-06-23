use crate::{schema::*, ExecutionOutcomeStatus};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use near_lake_framework::near_indexer_primitives::{
    views::{ExecutionOutcomeView, ReceiptView},
    CryptoHash,
};

#[derive(Identifiable, Insertable, Queryable)]
#[diesel(primary_key(receipt_id))]
pub struct ExecutionOutcome {
    pub receipt_id: String,
    pub block_hash: String,
    pub chunk_index: i32,
    pub timestamp: NaiveDateTime,
    pub gas_burnt: String,
    pub tokens_burnt: String,
    pub account_id: String,
    pub status: String,
    pub shard: String,
}

impl ExecutionOutcome {
    pub fn new(
        receipt: &ReceiptView,
        block_hash: CryptoHash,
        chunk_index: i32,
        timestamp: NaiveDateTime,
        outcome: &ExecutionOutcomeView,
        shard_id: u64,
    ) -> Self {
        Self {
            receipt_id: receipt.receipt_id.to_string(),
            block_hash: block_hash.to_string(),
            chunk_index,
            timestamp,
            gas_burnt: outcome.gas_burnt.to_string(),
            tokens_burnt: outcome.tokens_burnt.to_string(),
            account_id: outcome.executor_id.to_string(),
            status: ExecutionOutcomeStatus::from(outcome.status.clone()).to_string(),
            shard: shard_id.to_string(),
        }
    }
}
