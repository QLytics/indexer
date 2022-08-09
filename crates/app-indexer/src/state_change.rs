use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::{views::StateChangeWithCauseView, CryptoHash};
use qlytics_graphql::AccountChange;

pub fn handle_state_changes(
    state_changes: &[StateChangeWithCauseView],
    block_hash: CryptoHash,
    timestamp: NaiveDateTime,
) -> Vec<AccountChange> {
    state_changes
        .iter()
        .enumerate()
        .filter_map(|(index_in_block, state_change)| {
            AccountChange::new(state_change, block_hash, timestamp, index_in_block as i64)
        })
        .collect()
}
