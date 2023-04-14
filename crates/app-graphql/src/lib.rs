#[macro_use]
extern crate serde_json;

mod util;

use base64::{engine::general_purpose, Engine as _};
use graphql_client::GraphQLQuery;
use near_crypto::PublicKey;
use near_lake_framework::near_indexer_primitives::{
    types::AccountId,
    views::{
        AccountView, ActionView, BlockView, ExecutionOutcomeView, ExecutionStatusView,
        ReceiptEnumView, ReceiptView, SignedTransactionView, StateChangeCauseView,
        StateChangeValueView, StateChangeWithCauseView,
    },
    CryptoHash, IndexerChunkView,
};
use near_primitives::account::AccessKeyPermission as NearAccessKeyPermission;
use strum::{Display, EnumString};
use util::get_action_type_and_value;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/query.graphql",
    response_derives = "Debug"
)]
pub struct AddBlockData;

pub use add_block_data::{
    AccessKey, Account, AccountChange, ActionReceipt, ActionReceiptAction, ActionReceiptInputData,
    ActionReceiptOutputData, Block, BlockData, Chunk, DataReceipt, ExecutionOutcome,
    ExecutionOutcomeReceipt, Receipt, Transaction, TransactionAction,
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/query.graphql",
    response_derives = "Debug"
)]
pub struct AddGenesisBlockData;

pub use add_genesis_block_data::{
    AccessKey as GenesisAccessKey, Account as GenesisAccount, GenesisBlockData,
};
impl From<Account> for GenesisAccount {
    fn from(account: Account) -> Self {
        let Account {
            account_id,
            created_by_receipt_id,
            deleted_by_receipt_id,
            last_update_block_height,
        } = account;
        Self {
            account_id,
            created_by_receipt_id,
            deleted_by_receipt_id,
            last_update_block_height,
        }
    }
}

impl From<AccessKey> for GenesisAccessKey {
    fn from(access_key: AccessKey) -> Self {
        let AccessKey {
            public_key,
            account_id,
            created_by_receipt_id,
            deleted_by_receipt_id,
            permission_kind,
            last_update_block_height,
        } = access_key;
        Self {
            public_key,
            account_id,
            created_by_receipt_id,
            deleted_by_receipt_id,
            permission_kind,
            last_update_block_height,
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.graphql",
    query_path = "src/graphql/query.graphql",
    response_derives = "Debug"
)]
pub struct DeleteAccounts;

impl add_block_data::Block {
    pub fn new(block_view: &BlockView, timestamp: i64) -> Self {
        Self {
            block_hash: block_view.header.hash.to_string(),
            block_height: block_view.header.height.to_string(),
            prev_block_hash: block_view.header.prev_hash.to_string(),
            block_timestamp: timestamp.to_string(),
            total_supply: block_view.header.total_supply.to_string(),
            gas_price: block_view.header.gas_price.to_string(),
            author_account_id: block_view.author.to_string(),
        }
    }
}

impl add_block_data::Chunk {
    pub fn new(chunk_view: &IndexerChunkView, block_hash: CryptoHash) -> Self {
        Self {
            chunk_hash: chunk_view.header.chunk_hash.to_string(),
            included_in_block_hash: block_hash.to_string(),
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
        index_in_chunk: i64,
        block_timestamp: i64,
        outcome: &ExecutionOutcomeView,
    ) -> Self {
        Self {
            transaction_hash: transaction.hash.to_string(),
            included_in_block_hash: block_hash.to_string(),
            included_in_chunk_hash: chunk_hash.to_string(),
            index_in_chunk,
            block_timestamp: block_timestamp.to_string(),
            signer_account_id: transaction.signer_id.to_string(),
            signer_public_key: transaction.public_key.to_string(),
            nonce: transaction.nonce.to_string(),
            receiver_account_id: transaction.receiver_id.to_string(),
            signature: transaction.signature.to_string(),
            status: ExecutionOutcomeStatus::from(outcome.status.clone()).to_string(),
            converted_into_receipt_id: outcome.receipt_ids.first().unwrap().to_string(),
            receipt_conversion_gas_burnt: outcome.gas_burnt.to_string(),
            receipt_conversion_tokens_burnt: outcome.tokens_burnt.to_string(),
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
        index_in_transaction: i64,
        action_view: &ActionView,
    ) -> Self {
        let (action_kind, args) = get_action_type_and_value(action_view);
        Self {
            transaction_hash: transaction.hash.to_string(),
            index_in_transaction,
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
    Delegate,
    DeleteKey,
    DeleteAccount,
}

impl add_block_data::Receipt {
    pub fn new(
        receipt: &ReceiptView,
        block_hash: CryptoHash,
        chunk_hash: CryptoHash,
        index_in_chunk: i64,
        timestamp: i64,
        transaction_hash: CryptoHash,
    ) -> Self {
        Self {
            receipt_id: receipt.receipt_id.to_string(),
            included_in_block_hash: block_hash.to_string(),
            included_in_chunk_hash: chunk_hash.to_string(),
            index_in_chunk,
            included_in_block_timestamp: timestamp.to_string(),
            predecessor_account_id: receipt.predecessor_id.to_string(),
            receiver_account_id: receipt.receiver_id.to_string(),
            receipt_kind: match receipt.receipt {
                ReceiptEnumView::Action { .. } => ReceiptKind::Action.to_string(),
                ReceiptEnumView::Data { .. } => ReceiptKind::Data.to_string(),
            },
            originated_from_transaction_hash: transaction_hash.to_string(),
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
            data_base64: data.map(|d| general_purpose::STANDARD.encode(d)),
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

impl add_block_data::ActionReceiptAction {
    pub fn new(
        receipt: &ReceiptView,
        index: i64,
        action_view: &ActionView,
        timestamp: i64,
    ) -> Self {
        let (action_kind, args) = get_action_type_and_value(action_view);
        Self {
            receipt_id: receipt.receipt_id.to_string(),
            index_in_action_receipt: index,
            action_kind: action_kind.to_string(),
            args: args.to_string(),
            predecessor_id: receipt.predecessor_id.to_string(),
            receiver_id: receipt.receiver_id.to_string(),
            timestamp: timestamp.to_string(),
        }
    }
}

impl add_block_data::ActionReceiptInputData {
    pub fn new(data_id: CryptoHash, receipt_id: CryptoHash) -> Self {
        Self {
            data_id: data_id.to_string(),
            receipt_id: receipt_id.to_string(),
        }
    }
}

impl add_block_data::ActionReceiptOutputData {
    pub fn new(data_id: CryptoHash, receipt_id: CryptoHash, receiver_id: &AccountId) -> Self {
        Self {
            data_id: data_id.to_string(),
            receipt_id: receipt_id.to_string(),
            receiver_id: receiver_id.to_string(),
        }
    }
}

impl add_block_data::ExecutionOutcome {
    pub fn new(
        receipt: &ReceiptView,
        block_hash: CryptoHash,
        chunk_index: i64,
        timestamp: i64,
        outcome: &ExecutionOutcomeView,
        shard_id: u64,
    ) -> Self {
        Self {
            receipt_id: receipt.receipt_id.to_string(),
            block_hash: block_hash.to_string(),
            chunk_index,
            timestamp: timestamp.to_string(),
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

impl add_block_data::Account {
    pub fn new(
        account_id: &AccountId,
        created_by_receipt_id: Option<&CryptoHash>,
        block_height: u64,
    ) -> Self {
        Self {
            account_id: account_id.to_string(),
            created_by_receipt_id: created_by_receipt_id.map(CryptoHash::to_string),
            deleted_by_receipt_id: None,
            last_update_block_height: block_height.to_string(),
        }
    }
}

impl add_block_data::AccountChange {
    pub fn new(
        state_change_with_cause: &StateChangeWithCauseView,
        block_hash: CryptoHash,
        timestamp: i64,
        index_in_block: i64,
    ) -> Option<Self> {
        let StateChangeWithCauseView { cause, value } = state_change_with_cause;

        let (account_id, account): (String, Option<&AccountView>) = match value {
            StateChangeValueView::AccountUpdate {
                account_id,
                account,
            } => (account_id.to_string(), Some(account)),
            StateChangeValueView::AccountDeletion { account_id } => (account_id.to_string(), None),
            _ => return None,
        };

        Some(Self {
            account_id,
            timestamp: timestamp.to_string(),
            block_hash: block_hash.to_string(),
            transaction_hash: if let StateChangeCauseView::TransactionProcessing { tx_hash } = cause
            {
                Some(tx_hash.to_string())
            } else {
                None
            },
            receipt_id: match cause {
                StateChangeCauseView::ActionReceiptProcessingStarted { receipt_hash } => {
                    Some(receipt_hash.to_string())
                }
                StateChangeCauseView::ActionReceiptGasReward { receipt_hash } => {
                    Some(receipt_hash.to_string())
                }
                StateChangeCauseView::ReceiptProcessing { receipt_hash } => {
                    Some(receipt_hash.to_string())
                }
                StateChangeCauseView::PostponedReceipt { receipt_hash } => {
                    Some(receipt_hash.to_string())
                }
                _ => None,
            },
            update_reason: UpdateReason::from(cause).to_string(),
            nonstaked_balance: if let Some(acc) = account {
                acc.amount.to_string()
            } else {
                "0".to_string()
            },
            staked_balance: if let Some(acc) = account {
                acc.locked.to_string()
            } else {
                "0".to_string()
            },
            storage_usage: if let Some(acc) = account {
                acc.storage_usage.to_string()
            } else {
                "0".to_string()
            },
            index_in_block,
        })
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum UpdateReason {
    NotWritableToDisk,
    InitialState,
    TransactionProcessing,
    ActionReceiptProcessingStarted,
    ActionReceiptGasReward,
    ReceiptProcessing,
    PostponedReceipt,
    UpdatedDelayedReceipts,
    ValidatorAccountsUpdate,
    Migration,
    Resharding,
}

impl From<&StateChangeCauseView> for UpdateReason {
    fn from(state_change_cause: &StateChangeCauseView) -> Self {
        match state_change_cause {
            StateChangeCauseView::NotWritableToDisk => Self::NotWritableToDisk,
            StateChangeCauseView::InitialState => Self::InitialState,
            StateChangeCauseView::TransactionProcessing { .. } => Self::TransactionProcessing,
            StateChangeCauseView::ActionReceiptProcessingStarted { .. } => {
                Self::ActionReceiptProcessingStarted
            }
            StateChangeCauseView::ActionReceiptGasReward { .. } => Self::ActionReceiptGasReward,
            StateChangeCauseView::ReceiptProcessing { .. } => Self::ReceiptProcessing,
            StateChangeCauseView::PostponedReceipt { .. } => Self::PostponedReceipt,
            StateChangeCauseView::UpdatedDelayedReceipts => Self::UpdatedDelayedReceipts,
            StateChangeCauseView::ValidatorAccountsUpdate => Self::ValidatorAccountsUpdate,
            StateChangeCauseView::Migration => Self::Migration,
            StateChangeCauseView::Resharding => Self::Resharding,
        }
    }
}

impl add_block_data::AccessKey {
    pub fn new(
        public_key: &PublicKey,
        account_id: &AccountId,
        permission: &NearAccessKeyPermission,
        created_by_receipt_id: Option<CryptoHash>,
        block_height: u64,
    ) -> Self {
        Self {
            public_key: public_key.to_string(),
            account_id: account_id.to_string(),
            created_by_receipt_id: created_by_receipt_id.map(|receipt_id| receipt_id.to_string()),
            deleted_by_receipt_id: None,
            permission_kind: AccessKeyPermission::from(permission).to_string(),
            last_update_block_height: block_height.to_string(),
        }
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessKeyPermission {
    FullAccess,
    FunctionCall,
}

impl From<&NearAccessKeyPermission> for AccessKeyPermission {
    fn from(permission: &NearAccessKeyPermission) -> Self {
        match permission {
            NearAccessKeyPermission::FunctionCall { .. } => Self::FunctionCall,
            NearAccessKeyPermission::FullAccess => Self::FullAccess,
        }
    }
}
