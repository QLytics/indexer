use crate::{schema::*, util::escape_json};
use diesel::prelude::*;
use near_lake_framework::near_indexer_primitives::views::{ActionView, SignedTransactionView};

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
