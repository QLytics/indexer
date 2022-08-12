use near_lake_framework::near_indexer_primitives::{
    types::AccountId,
    views::{ActionView, ExecutionStatusView, ReceiptEnumView},
    IndexerExecutionOutcomeWithReceipt,
};
use qlytics_graphql::{AccessKey, Account};
use rayon::prelude::*;

pub fn handle_accounts(
    outcomes: &[IndexerExecutionOutcomeWithReceipt],
    block_height: u64,
) -> Vec<(Option<Account>, Option<AccountId>, Option<AccessKey>)> {
    outcomes
        .par_iter()
        .filter(|outcome| {
            matches!(
                outcome.execution_outcome.outcome.status,
                ExecutionStatusView::SuccessValue(_) | ExecutionStatusView::SuccessReceiptId(_)
            )
        })
        .filter_map(|outcome| {
            if let ReceiptEnumView::Action { actions, .. } = &outcome.receipt.receipt {
                Some(actions.into_par_iter().map(
                    |action| -> (Option<Account>, Option<AccountId>, Option<AccessKey>) {
                        match action {
                            ActionView::CreateAccount => (
                                Some(Account::new(
                                    &outcome.receipt.receiver_id,
                                    Some(&outcome.receipt.receipt_id),
                                    block_height,
                                )),
                                None,
                                None,
                            ),
                            ActionView::Transfer { .. } => {
                                if outcome.receipt.receiver_id.len() == 64usize {
                                    (
                                        Some(Account::new(
                                            &outcome.receipt.receiver_id,
                                            Some(&outcome.receipt.receipt_id),
                                            block_height,
                                        )),
                                        None,
                                        None,
                                    )
                                } else {
                                    (None, None, None)
                                }
                            }
                            ActionView::DeleteAccount { .. } => {
                                (None, Some(outcome.receipt.receiver_id.clone()), None)
                            }
                            ActionView::AddKey {
                                public_key,
                                access_key,
                            } => (
                                None,
                                None,
                                Some(AccessKey::new(
                                    public_key,
                                    &outcome.receipt.receiver_id,
                                    &access_key.permission.clone().into(),
                                    Some(outcome.receipt.receipt_id),
                                    block_height,
                                )),
                            ),
                            _ => (None, None, None),
                        }
                    },
                ))
            } else {
                None
            }
        })
        .flatten()
        .collect()
}
