use chrono::NaiveDateTime;
use near_lake_framework::near_indexer_primitives::{
    views::ReceiptEnumView, CryptoHash, IndexerChunkView, IndexerShard, StreamerMessage,
};
use near_ql_db::{ExecutionOutcome, Receipt};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::{collections::HashMap, sync::Arc};

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_chunk_receipts(
    shard: &IndexerShard,
    chunk: &IndexerChunkView,
    block_hash: CryptoHash,
    chunk_hash: CryptoHash,
    timestamp: NaiveDateTime,
    receipts: &Arc<RwLock<Vec<Receipt>>>,
    execution_outcomes: &Arc<RwLock<Vec<ExecutionOutcome>>>,
    receipt_id_to_tx_hash: &Arc<RwLock<HashMap<CryptoHash, CryptoHash>>>,
    data_id_to_tx_hash: &Arc<RwLock<HashMap<CryptoHash, CryptoHash>>>,
    misses: &Arc<RwLock<u32>>,
) {
    chunk
        .receipts
        .par_iter()
        .enumerate()
        .for_each(|(chunk_index, receipt_view)| {
            if let Some(outcome) = shard
                .receipt_execution_outcomes
                .iter()
                .find(|r| r.execution_outcome.id == receipt_view.receipt_id)
            {
                let execution_outcome = ExecutionOutcome::new(
                    receipt_view,
                    block_hash,
                    chunk_index as i32,
                    timestamp,
                    &outcome.execution_outcome.outcome,
                    shard.shard_id,
                );
                execution_outcomes.write().push(execution_outcome);
            }

            let removed_receipt = receipt_id_to_tx_hash
                .write()
                .get(&receipt_view.receipt_id)
                .cloned();
            if let Some(tx_hash) = removed_receipt {
                let receipt = Receipt::new(
                    receipt_view,
                    block_hash,
                    chunk_hash,
                    chunk_index as i32,
                    timestamp,
                    tx_hash,
                );
                receipts.write().push(receipt);

                if let ReceiptEnumView::Action {
                    output_data_receivers,
                    ..
                } = &receipt_view.receipt
                {
                    output_data_receivers.iter().for_each(|receiver| {
                        data_id_to_tx_hash.write().insert(receiver.data_id, tx_hash);
                    });
                }
            } else {
                let mut misses = misses.write();
                *misses += 1;
            }
        });
}

pub(crate) fn handle_shard_receipts(
    msg: &StreamerMessage,
    receipt_id_to_tx_hash: &Arc<RwLock<HashMap<CryptoHash, CryptoHash>>>,
) {
    let mut outcome_receipts: Vec<_> = msg
        .shards
        .iter()
        .flat_map(|shard| {
            shard.receipt_execution_outcomes.iter().flat_map(|outcome| {
                outcome
                    .execution_outcome
                    .outcome
                    .receipt_ids
                    .iter()
                    .map(|receipt_id| (outcome.execution_outcome.id, *receipt_id))
            })
        })
        .collect();
    let mut retries_left = 5;
    loop {
        outcome_receipts.drain_filter(|(executed_receipt_id, produced_receipt_id)| {
            let mut receipt_id_to_tx_hash = receipt_id_to_tx_hash.write();
            if let Some(tx_hash) = receipt_id_to_tx_hash.get(executed_receipt_id).cloned() {
                receipt_id_to_tx_hash.insert(*produced_receipt_id, tx_hash);
                true
            } else {
                false
            }
        });
        retries_left -= 1;
        if retries_left == 0 || outcome_receipts.is_empty() {
            break;
        }
    }
}