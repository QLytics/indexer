use crate::{schema::*, ExecutionOutcomeStatus};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use near_lake_framework::near_indexer_primitives::{
    views::{ExecutionOutcomeView, SignedTransactionView},
    CryptoHash,
};

#[derive(Identifiable, Insertable, Queryable)]
#[diesel(primary_key(hash))]
pub struct Transaction {
    pub hash: String,
    pub block_hash: String,
    pub chunk_hash: String,
    pub chunk_index: i32,
    pub timestamp: NaiveDateTime,
    pub signer_id: String,
    pub public_key: String,
    pub nonce: String,
    pub receiver_id: String,
    pub signature: String,
    pub status: String,
    pub receipt_id: String,
    pub gas_burnt: String,
    pub tokens_burnt: String,
}

impl Transaction {
    pub fn new(
        transaction: &SignedTransactionView,
        block_hash: CryptoHash,
        chunk_hash: CryptoHash,
        chunk_index: i32,
        timestamp: NaiveDateTime,
        outcome: &ExecutionOutcomeView,
    ) -> Self {
        Self {
            hash: transaction.hash.to_string(),
            block_hash: block_hash.to_string(),
            chunk_hash: chunk_hash.to_string(),
            chunk_index,
            timestamp,
            signer_id: transaction.signer_id.to_string(),
            public_key: transaction.public_key.to_string(),
            nonce: transaction.nonce.to_string(),
            receiver_id: transaction.receiver_id.to_string(),
            signature: transaction.signature.to_string(),
            status: ExecutionOutcomeStatus::from(outcome.status.clone()).to_string(),
            receipt_id: outcome.receipt_ids.first().unwrap().to_string(),
            gas_burnt: outcome.gas_burnt.to_string(),
            tokens_burnt: outcome.tokens_burnt.to_string(),
        }
    }
}
