#[macro_use]
extern crate serde_json;

mod util;

use chrono::NaiveDateTime;
use graphql_client::GraphQLQuery;
use near_crypto::PublicKey;
use near_lake_framework::near_indexer_primitives::{
    types::AccountId,
    views::{
        ActionView, BlockView, ExecutionOutcomeView, ExecutionStatusView, ReceiptEnumView,
        ReceiptView, SignedTransactionView,
    },
    CryptoHash, IndexerChunkView,
};
use strum::{Display, EnumString};
use util::escape_json;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/query.graphql",
    response_derives = "Debug"
)]
pub struct AddBlockData;

pub use add_block_data::{
    ActionReceipt, Block, BlockData, Chunk, DataReceipt, ExecutionOutcome, ExecutionOutcomeReceipt,
    Receipt, Transaction, TransactionAction,
};

impl add_block_data::Block {
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

impl add_block_data::Chunk {
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

impl add_block_data::Transaction {
    pub fn new(
        transaction: &SignedTransactionView,
        block_hash: CryptoHash,
        chunk_hash: CryptoHash,
        chunk_index: i64,
        timestamp: NaiveDateTime,
        outcome: &ExecutionOutcomeView,
    ) -> Self {
        Self {
            hash: transaction.hash.to_string(),
            block_hash: block_hash.to_string(),
            chunk_hash: chunk_hash.to_string(),
            chunk_index,
            timestamp: timestamp.timestamp().to_string(),
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

impl From<ExecutionStatusView> for ExecutionOutcomeStatus {
    fn from(status: ExecutionStatusView) -> Self {
        match status {
            ExecutionStatusView::Unknown => ExecutionOutcomeStatus::Unknown,
            ExecutionStatusView::Failure(_) => ExecutionOutcomeStatus::Failure,
            ExecutionStatusView::SuccessValue(_) => ExecutionOutcomeStatus::SuccessValue,
            ExecutionStatusView::SuccessReceiptId(_) => ExecutionOutcomeStatus::SuccessReceiptId,
        }
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionOutcomeStatus {
    Unknown,
    Failure,
    SuccessValue,
    SuccessReceiptId,
}

impl add_block_data::TransactionAction {
    pub fn new(
        transaction: &SignedTransactionView,
        transaction_index: i64,
        action_view: &ActionView,
    ) -> Self {
        let (action_kind, args) = match action_view {
            ActionView::CreateAccount => (ActionKind::CreateAccount, json!({})),
            ActionView::DeployContract { code } => (
                ActionKind::DeployContract,
                json!({
                    "code_sha256":  hex::encode(
                        base64::decode(code).expect("code expected to be encoded to base64")
                    )
                }),
            ),
            ActionView::FunctionCall {
                method_name,
                args,
                gas,
                deposit,
            } => {
                let mut arguments = json!({
                    "method_name": method_name.escape_default().to_string(),
                    "args_base64": args,
                    "gas": gas,
                    "deposit": deposit.to_string(),
                });
                if let Ok(decoded_args) = base64::decode(args) {
                    if let Ok(mut args_json) = serde_json::from_slice(&decoded_args) {
                        escape_json(&mut args_json);
                        arguments["args_json"] = args_json;
                    }
                }
                (ActionKind::FunctionCall, arguments)
            }
            ActionView::Transfer { deposit } => (
                ActionKind::Transfer,
                json!({ "deposit": deposit.to_string() }),
            ),
            ActionView::Stake { stake, public_key } => (
                ActionKind::Stake,
                json!({
                    "stake": stake.to_string(),
                    "public_key": public_key,
                }),
            ),
            ActionView::AddKey {
                public_key,
                access_key,
            } => (
                ActionKind::AddKey,
                json!({
                    "public_key": public_key,
                    "access_key": access_key,
                }),
            ),
            ActionView::DeleteKey { public_key } => (
                ActionKind::DeleteKey,
                json!({
                    "public_key": public_key,
                }),
            ),
            ActionView::DeleteAccount { beneficiary_id } => (
                ActionKind::DeleteAccount,
                json!({
                    "beneficiary_id": beneficiary_id,
                }),
            ),
        };
        Self {
            hash: transaction.hash.to_string(),
            transaction_index,
            action_kind: action_kind.to_string(),
            args: args.to_string(),
        }
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionKind {
    CreateAccount,
    DeployContract,
    FunctionCall,
    Transfer,
    Stake,
    AddKey,
    DeleteKey,
    DeleteAccount,
}

impl add_block_data::Receipt {
    pub fn new(
        receipt: &ReceiptView,
        block_hash: CryptoHash,
        chunk_hash: CryptoHash,
        chunk_index: i64,
        timestamp: String,
        transaction_hash: CryptoHash,
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
            transaction_hash: transaction_hash.to_string(),
        }
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ReceiptKind {
    Action,
    Data,
}

impl add_block_data::DataReceipt {
    pub fn new(data_id: CryptoHash, receipt_id: CryptoHash, data: Option<Vec<u8>>) -> Self {
        Self {
            data_id: data_id.to_string(),
            receipt_id: receipt_id.to_string(),
            data_base64: data.map(base64::encode),
        }
    }
}

impl add_block_data::ActionReceipt {
    pub fn new(
        receipt_id: CryptoHash,
        signer_account_id: &AccountId,
        signer_public_key: &PublicKey,
        gas_price: String,
    ) -> Self {
        Self {
            receipt_id: receipt_id.to_string(),
            signer_account_id: signer_account_id.to_string(),
            signer_public_key: signer_public_key.to_string(),
            gas_price,
        }
    }
}

impl add_block_data::ExecutionOutcome {
    pub fn new(
        receipt: &ReceiptView,
        block_hash: CryptoHash,
        chunk_index: i64,
        timestamp: String,
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

impl add_block_data::ExecutionOutcomeReceipt {
    pub fn new(
        receipt_id: CryptoHash,
        index_in_execution_outcome: i64,
        produced_receipt_id: CryptoHash,
    ) -> Self {
        Self {
            receipt_id: receipt_id.to_string(),
            index_in_execution_outcome,
            produced_receipt_id: produced_receipt_id.to_string(),
        }
    }
}
