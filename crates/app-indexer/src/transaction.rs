use near_lake_framework::near_indexer_primitives::{
    CryptoHash, IndexerChunkView, IndexerTransactionWithOutcome,
};
use qlytics_graphql::{Transaction, TransactionAction};
use rayon::prelude::*;

pub(crate) fn handle_transactions(
    chunk: &IndexerChunkView,
    chunk_hash: CryptoHash,
    block_hash: CryptoHash,
    timestamp: i64,
) -> (Vec<Transaction>, Vec<TransactionAction>) {
    let (transactions, transaction_actions): (Vec<_>, Vec<_>) = chunk
        .transactions
        .par_iter()
        .enumerate()
        .map(
            |(
                chunk_index,
                IndexerTransactionWithOutcome {
                    transaction,
                    outcome,
                },
            )| {
                let transaction_actions: Vec<_> = transaction
                    .actions
                    .par_iter()
                    .enumerate()
                    .map(|(transaction_index, action_view)| {
                        TransactionAction::new(transaction, transaction_index as i64, action_view)
                    })
                    .collect();

                (
                    Transaction::new(
                        transaction,
                        block_hash,
                        chunk_hash,
                        chunk_index as i64,
                        timestamp,
                        &outcome.execution_outcome.outcome,
                    ),
                    transaction_actions,
                )
            },
        )
        .unzip();
    (
        transactions,
        transaction_actions.into_iter().flatten().collect(),
    )
}
