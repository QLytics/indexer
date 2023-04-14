use base64::{engine::general_purpose, Engine as _};
use near_lake_framework::near_indexer_primitives::views::ActionView;

use crate::ActionKind;

pub(crate) fn get_action_type_and_value(
    action_view: &ActionView,
) -> (ActionKind, serde_json::Value) {
    match action_view {
        ActionView::CreateAccount => (ActionKind::CreateAccount, json!({})),
        ActionView::DeployContract { code } => (
            ActionKind::DeployContract,
            json!({
                "code_sha256":  hex::encode(
                    general_purpose::STANDARD.decode(code).expect("code expected to be encoded to base64")
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
            if let Ok(decoded_args) = general_purpose::STANDARD.decode(args) {
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
        ActionView::Delegate {
            delegate_action,
            signature,
        } => (
            ActionKind::Delegate,
            json!({
                "delegate_action": delegate_action,
                "signature": signature
            }),
        ),
    }
}

pub(crate) fn escape_json(object: &mut serde_json::Value) {
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
