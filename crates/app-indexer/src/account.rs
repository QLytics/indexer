use either::Either;
use near_lake_framework::near_indexer_primitives::{
    types::AccountId,
    views::{ActionView, ExecutionStatusView, ReceiptEnumView},
    IndexerExecutionOutcomeWithReceipt,
};
use qlytics_graphql::Account;
use rayon::prelude::*;

pub fn handle_accounts(
    outcomes: &[IndexerExecutionOutcomeWithReceipt],
    block_height: u64,
) -> Vec<Either<Account, AccountId>> {
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
                Some(actions.into_par_iter().filter_map(|action| match action {
                    ActionView::CreateAccount => Some(Either::Left(Account::new(
                        &outcome.receipt.receiver_id,
                        Some(&outcome.receipt.receipt_id),
                        block_height,
                    ))),
                    ActionView::Transfer { .. } => {
                        if outcome.receipt.receiver_id.len() == 64usize {
                            Some(Either::Left(Account::new(
                                &outcome.receipt.receiver_id,
                                Some(&outcome.receipt.receipt_id),
                                block_height,
                            )))
                        } else {
                            None
                        }
                    }
                    ActionView::DeleteAccount { .. } => {
                        Some(Either::Right(outcome.receipt.receiver_id.clone()))
                    }
                    _ => None,
                }))
            } else {
                None
            }
        })
        .flatten()
        .collect()
}
