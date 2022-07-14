use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::{
    CryptoHash, IndexerChunkView, IndexerTransactionWithOutcome,
};
use parking_lot::RwLock;
use qlytics_db::{Transaction, TransactionAction};
use rayon::prelude::*;
use std::sync::Arc;

pub(crate) fn handle_transactions(
    chunk: &IndexerChunkView,
    chunk_hash: CryptoHash,
    block_hash: CryptoHash,
    timestamp: NaiveDateTime,
    transactions: &Arc<RwLock<Vec<Transaction>>>,
    transaction_actions: &Arc<RwLock<Vec<TransactionAction>>>,
) {
    chunk.transactions.par_iter().enumerate().for_each(
        |(
            chunk_index,
            IndexerTransactionWithOutcome {
                transaction,
                outcome,
            },
        )| {
            transaction.actions.par_iter().enumerate().for_each(
                |(transaction_index, action_view)| {
                    let transaction_action =
                        TransactionAction::new(transaction, transaction_index as i32, action_view);
                    transaction_actions.write().push(transaction_action);
                },
            );

            let transaction = Transaction::new(
                transaction,
                block_hash,
                chunk_hash,
                chunk_index as i32,
                timestamp,
                &outcome.execution_outcome.outcome,
            );
            transactions.write().push(transaction);
        },
    );
}
