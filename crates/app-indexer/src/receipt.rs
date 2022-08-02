use chrono::NaiveDateTime;
use itertools::Itertools;
use near_lake_framework::near_indexer_primitives::{
    views::ReceiptEnumView, CryptoHash, IndexerChunkView, IndexerShard, StreamerMessage,
};
use parking_lot::RwLock;
use qlytics_graphql::{
    ActionReceipt, ActionReceiptAction, ActionReceiptInputData, DataReceipt, ExecutionOutcome,
    ExecutionOutcomeReceipt, Receipt,
};
use rayon::prelude::*;
use std::{collections::HashMap, sync::Arc};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn handle_chunk_receipts(
    shard: &IndexerShard,
    chunk: &IndexerChunkView,
    block_hash: CryptoHash,
    chunk_hash: CryptoHash,
    timestamp: NaiveDateTime,
    receipt_id_to_tx_hash: &Arc<RwLock<HashMap<CryptoHash, (CryptoHash, u8)>>>,
    data_id_to_tx_hash: &Arc<RwLock<HashMap<CryptoHash, CryptoHash>>>,
    misses: &Arc<RwLock<u32>>,
) -> (
    Vec<Receipt>,
    Vec<DataReceipt>,
    Vec<ActionReceipt>,
    Vec<ActionReceiptAction>,
    Vec<ActionReceiptInputData>,
    Vec<ExecutionOutcome>,
    Vec<ExecutionOutcomeReceipt>,
) {
    let (
        receipts,
        data_receipts,
        action_receipts,
        action_receipt_actions,
        action_receipt_input_datas,
        execution_outcome,
        execution_outcome_receipts,
    ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = chunk
        .receipts
        .par_iter()
        .enumerate()
        .map(|(chunk_index, receipt_view)| {
            let (execution_outcome, execution_outcome_receipts) = if let Some(outcome) = shard
                .receipt_execution_outcomes
                .iter()
                .find(|r| r.execution_outcome.id == receipt_view.receipt_id)
            {
                let execution_outcome = Some(ExecutionOutcome::new(
                    receipt_view,
                    block_hash,
                    chunk_index as i64,
                    timestamp.timestamp().to_string(),
                    &outcome.execution_outcome.outcome,
                    shard.shard_id,
                ));

                let execution_outcome_receipts = outcome
                    .execution_outcome
                    .outcome
                    .receipt_ids
                    .iter()
                    .enumerate()
                    .map(|(index, receipt_id)| {
                        ExecutionOutcomeReceipt::new(
                            outcome.execution_outcome.id,
                            index as i64,
                            *receipt_id,
                        )
                    })
                    .collect();
                (execution_outcome, execution_outcome_receipts)
            } else {
                (None, vec![])
            };

            let data_receipt =
                if let ReceiptEnumView::Data { data_id, data } = &receipt_view.receipt {
                    Some(DataReceipt::new(
                        *data_id,
                        receipt_view.receipt_id,
                        data.clone(),
                    ))
                } else {
                    None
                };
            let (action_receipt, action_receipt_actions, action_receipt_input_datas) =
                if let ReceiptEnumView::Action {
                    signer_id,
                    signer_public_key,
                    gas_price,
                    actions,
                    input_data_ids,
                    ..
                } = &receipt_view.receipt
                {
                    let action_receipt = ActionReceipt::new(
                        receipt_view.receipt_id,
                        signer_id,
                        signer_public_key,
                        gas_price.to_string(),
                    );
                    let action_receipt_actions: Vec<_> = actions
                        .iter()
                        .enumerate()
                        .map(|(index, action_view)| {
                            ActionReceiptAction::new(
                                receipt_view,
                                index as i64,
                                action_view,
                                timestamp.timestamp().to_string(),
                            )
                        })
                        .collect();
                    let action_receipt_input_datas: Vec<_> = input_data_ids
                        .iter()
                        .map(|data_id| {
                            ActionReceiptInputData::new(receipt_view.receipt_id, *data_id)
                        })
                        .collect();
                    (
                        Some(action_receipt),
                        Some(action_receipt_actions),
                        Some(action_receipt_input_datas),
                    )
                } else {
                    (None, None, None)
                };

            let tx_hash = receipt_id_to_tx_hash
                .write()
                .get(&receipt_view.receipt_id)
                .cloned();
            let receipt = if let Some((tx_hash, _)) = tx_hash {
                let receipt = Receipt::new(
                    receipt_view,
                    block_hash,
                    chunk_hash,
                    chunk_index as i64,
                    timestamp.timestamp().to_string(),
                    tx_hash,
                );

                if let ReceiptEnumView::Action {
                    output_data_receivers,
                    ..
                } = &receipt_view.receipt
                {
                    output_data_receivers.iter().for_each(|receiver| {
                        data_id_to_tx_hash.write().insert(receiver.data_id, tx_hash);
                    });
                }
                Some(receipt)
            } else {
                let mut misses = misses.write();
                *misses += 1;
                None
            };
            (
                receipt,
                data_receipt,
                action_receipt,
                action_receipt_actions,
                action_receipt_input_datas,
                execution_outcome,
                execution_outcome_receipts,
            )
        })
        .collect::<Vec<_>>()
        .into_iter()
        .multiunzip();
    (
        receipts.into_iter().flatten().collect(),
        data_receipts.into_iter().flatten().collect(),
        action_receipts.into_iter().flatten().collect(),
        action_receipt_actions
            .into_iter()
            .flatten()
            .flatten()
            .collect(),
        action_receipt_input_datas
            .into_iter()
            .flatten()
            .flatten()
            .collect(),
        execution_outcome.into_iter().flatten().collect(),
        execution_outcome_receipts.into_iter().flatten().collect(),
    )
}

pub(crate) fn handle_shard_receipts(
    msg: &StreamerMessage,
    receipt_id_to_tx_hash: &Arc<RwLock<HashMap<CryptoHash, (CryptoHash, u8)>>>,
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
