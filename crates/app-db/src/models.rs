use crate::schema::*;
use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::{
    views::{ActionView, ExecutionOutcomeView, ExecutionStatusView, SignedTransactionView},
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transaction: SignedTransactionView,
        block_hash: CryptoHash,
        chunk_hash: CryptoHash,
        chunk_index: i32,
        timestamp: NaiveDateTime,
        outcome: ExecutionOutcomeView,
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
            status: ExecutionOutcomeStatus::from(outcome.status).to_string(),
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

#[derive(Identifiable, Insertable, Queryable)]
#[diesel(primary_key(hash))]
pub struct TransactionAction {
    pub hash: String,
    pub transaction_index: i32,
    pub action_kind: String,
    pub args: String,
}

impl TransactionAction {
    pub fn new(
        transaction: &SignedTransactionView,
        transaction_index: i32,
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

fn escape_json(object: &mut serde_json::Value) {
    match object {
        serde_json::Value::Object(ref mut value) => {
            for (_key, val) in value {
                escape_json(val);
            }
        }
        serde_json::Value::Array(ref mut values) => {
            for element in values.iter_mut() {
                escape_json(element)
            }
        }
        serde_json::Value::String(ref mut value) => *value = value.escape_default().to_string(),
        _ => {}
    }
}
