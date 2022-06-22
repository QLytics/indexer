use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use near_lake_framework::near_indexer_primitives::{
    views::{ReceiptEnumView, ReceiptView},
    CryptoHash,
};

#[derive(Identifiable, Insertable, Queryable)]
#[diesel(primary_key(receipt_id))]
pub struct Receipt {
    pub receipt_id: String,
    pub block_hash: String,
    pub chunk_hash: String,
    pub chunk_index: i32,
    pub timestamp: NaiveDateTime,
    pub predecessor_id: String,
    pub receiver_id: String,
    pub receipt_kind: String,
}

impl Receipt {
    pub fn new(
        receipt: ReceiptView,
        block_hash: CryptoHash,
        chunk_hash: CryptoHash,
        chunk_index: i32,
        timestamp: NaiveDateTime,
    ) -> Self {
        Self {
            receipt_id: receipt.receipt_id.to_string(),
            block_hash: block_hash.to_string(),
            chunk_hash: chunk_hash.to_string(),
            chunk_index,
            timestamp,
            predecessor_id: receipt.predecessor_id.to_string(),
            receiver_id: receipt.receiver_id.to_string(),
            receipt_kind: match receipt.receipt {
                ReceiptEnumView::Action { .. } => ReceiptKind::Action.to_string(),
                ReceiptEnumView::Data { .. } => ReceiptKind::Data.to_string(),
            },
        }
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ReceiptKind {
    Action,
    Data,
}
