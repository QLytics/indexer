#![feature(drain_filter)]

mod account;
mod genesis;
mod log;
mod receipt;
mod state_change;
mod transaction;

use account::handle_accounts;
use async_stream::try_stream;
use either::Either;
use futures_core::stream::Stream;
use genesis::handle_genesis;
use itertools::Itertools;
use near_jsonrpc_client::JsonRpcClient;
use near_lake_framework::{
    near_indexer_primitives::{
        types::AccountId, views::ReceiptEnumView, CryptoHash, StreamerMessage,
    },
    LakeConfigBuilder,
};
use parking_lot::RwLock;
use qlytics_core::Result;
use qlytics_graphql::{Block, BlockData, Chunk, GenesisBlockData};
use rayon::prelude::*;
use receipt::{handle_chunk_receipts, handle_shard_receipts};
use state_change::handle_state_changes;
use std::{
    collections::{HashMap, VecDeque},
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use transaction::handle_transactions;

pub async fn start_indexing(
) -> Result<impl Stream<Item = Result<Either<GenesisBlockData, (BlockData, Vec<AccountId>)>>>> {
    let start_block_height = env::var("START_BLOCK_HEIGHT")
        .map(|s| s.parse::<u64>().unwrap_or_default())
        .unwrap_or_default();
    let genesis_block_data = if start_block_height == 0 {
        let (accounts, access_keys) = handle_genesis().await?;
        Some(GenesisBlockData {
            accounts: accounts.into_iter().map(|a| a.into()).collect(),
            access_keys: access_keys.into_iter().map(|a| a.into()).collect(),
        })
    } else {
        None
    };

    let config = LakeConfigBuilder::default()
        .mainnet()
        .start_block_height(
            env::var("START_BLOCK_HEIGHT")
                .map(|s| s.parse::<u64>().unwrap_or_default())
                .unwrap_or_default(),
        )
        .build()
        // TODO: LakeConfigBuildError
        .unwrap();
    let client = Arc::new(JsonRpcClient::connect("https://rpc.mainnet.near.org"));

    let (_, mut stream) = near_lake_framework::streamer(config);

    let time = Arc::new(RwLock::new(Instant::now()));
    let eta = Arc::new(RwLock::new(VecDeque::new()));
    let receipt_id_to_tx_hash = Arc::new(RwLock::new(HashMap::new()));
    let data_id_to_tx_hash = Arc::new(RwLock::new(HashMap::new()));

    let misses = Arc::new(RwLock::new(0));

    Ok(try_stream! {
        if let Some(genesis_block_data) = genesis_block_data {
            yield Either::Left(genesis_block_data);
        }

        while let Some(msg) = stream.recv().await {
            let block_data = handle_streamer_message(
                client.clone(),
                msg,
                time.clone(),
                eta.clone(),
                receipt_id_to_tx_hash.clone(),
                data_id_to_tx_hash.clone(),
                misses.clone(),
            )
            .await?;

            yield Either::Right(block_data);

            receipt_id_to_tx_hash.write().retain(|_, (_, idx)| {
                *idx += 1;
                *idx < 15
            });
        }
    })
}

#[allow(clippy::too_many_arguments)]
async fn handle_streamer_message(
    client: Arc<JsonRpcClient>,
    msg: StreamerMessage,
    time: Arc<RwLock<Instant>>,
    eta: Arc<RwLock<VecDeque<(Duration, u64)>>>,
    receipt_id_to_tx_hash: Arc<RwLock<HashMap<CryptoHash, (CryptoHash, u8)>>>,
    data_id_to_tx_hash: Arc<RwLock<HashMap<CryptoHash, CryptoHash>>>,
    misses: Arc<RwLock<u32>>,
) -> Result<(BlockData, Vec<AccountId>)> {
    log::log(msg.block.header.height, &client, &time, &eta, &misses).await?;

    let block_hash = msg.block.header.hash;
    let timestamp = msg.block.header.timestamp_nanosec as i64 / 1_000_000;

    let block = Block::new(&msg.block, timestamp);

    msg.shards
        .par_iter()
        .filter_map(|shard| {
            shard.receipt_execution_outcomes.iter().for_each(|outcome| {
                outcome
                    .execution_outcome
                    .outcome
                    .receipt_ids
                    .iter()
                    .for_each(|receipt_id| {
                        let mut receipt_id_to_tx_hash = receipt_id_to_tx_hash.write();
                        if let Some(hash) = receipt_id_to_tx_hash
                            .get(&outcome.execution_outcome.id)
                            .cloned()
                        {
                            receipt_id_to_tx_hash.insert(*receipt_id, hash);
                        } else {
                            // eprintln!("Could not find parent for receipt {}", receipt_id);
                            // TODO strict mode
                        }
                    });
            });
            shard.chunk.as_ref()
        })
        .for_each(|chunk| {
            chunk.transactions.par_iter().for_each(|transaction| {
                if let Some(receipt) = &transaction.outcome.receipt {
                    receipt_id_to_tx_hash
                        .write()
                        .insert(receipt.receipt_id, (transaction.transaction.hash, 0));
                }
                transaction
                    .outcome
                    .execution_outcome
                    .outcome
                    .receipt_ids
                    .iter()
                    .for_each(|receipt_id| {
                        receipt_id_to_tx_hash
                            .write()
                            .insert(*receipt_id, (transaction.transaction.hash, 0));
                    });
            });
            chunk.receipts.iter().for_each(|receipt| {
                if let ReceiptEnumView::Data { data_id, .. } = receipt.receipt {
                    let removed_data_id = data_id_to_tx_hash.write().remove(&data_id);
                    if let Some(tx_hash) = removed_data_id {
                        receipt_id_to_tx_hash
                            .write()
                            .insert(receipt.receipt_id, (tx_hash, 0));
                    } else {
                        // TODO strict mode
                    }
                }
            });
        });

    #[allow(clippy::type_complexity)]
    let (
        chunks,
        transactions,
        transaction_actions,
        receipts,
        data_receipts,
        action_receipts,
        action_receipt_actions,
        action_receipt_input_datas,
        action_receipt_output_datas,
        execution_outcomes,
        execution_outcome_receipts,
        account_changes,
    ): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = msg
        .shards
        .par_iter()
        .filter_map(|shard| {
            let chunk_view = if let Some(chunk) = &shard.chunk {
                chunk
            } else {
                return None;
            };

            let chunk = Chunk::new(chunk_view, block_hash);

            let chunk_hash = chunk_view.header.chunk_hash;

            let (
                receipts,
                data_receipts,
                action_receipts,
                action_receipt_actions,
                action_receipt_input_datas,
                action_receipt_output_datas,
                execution_outcomes,
                execution_outcome_receipts,
            ) = handle_chunk_receipts(
                shard,
                chunk_view,
                block_hash,
                chunk_hash,
                timestamp,
                &receipt_id_to_tx_hash,
                &data_id_to_tx_hash,
                &misses,
            );

            let (transactions, transaction_actions) =
                handle_transactions(chunk_view, chunk_hash, block_hash, timestamp);

            let account_changes = handle_state_changes(&shard.state_changes, block_hash, timestamp);

            Some((
                chunk,
                transactions,
                transaction_actions,
                receipts,
                data_receipts,
                action_receipts,
                action_receipt_actions,
                action_receipt_input_datas,
                action_receipt_output_datas,
                execution_outcomes,
                execution_outcome_receipts,
                account_changes,
            ))
        })
        .collect::<Vec<_>>()
        .into_iter()
        .multiunzip();

    handle_shard_receipts(&msg, &receipt_id_to_tx_hash);

    let (accounts, account_ids, access_keys): (Vec<_>, Vec<_>, Vec<_>) = msg
        .shards
        .par_iter()
        .map(|shard| handle_accounts(&shard.receipt_execution_outcomes, msg.block.header.height))
        .flatten()
        .collect::<Vec<_>>()
        .into_iter()
        .multiunzip();

    Ok((
        BlockData {
            block,
            chunks,
            transactions: transactions.into_iter().flatten().collect(),
            transaction_actions: transaction_actions.into_iter().flatten().collect(),
            receipts: receipts.into_iter().flatten().collect(),
            data_receipts: data_receipts.into_iter().flatten().collect(),
            action_receipts: action_receipts.into_iter().flatten().collect(),
            action_receipt_actions: action_receipt_actions.into_iter().flatten().collect(),
            action_receipt_input_datas: action_receipt_input_datas.into_iter().flatten().collect(),
            action_receipt_output_datas: action_receipt_output_datas
                .into_iter()
                .flatten()
                .collect(),
            execution_outcomes: execution_outcomes.into_iter().flatten().collect(),
            execution_outcome_receipts: execution_outcome_receipts.into_iter().flatten().collect(),
            accounts: accounts.into_iter().flatten().collect(),
            account_changes: account_changes.into_iter().flatten().collect(),
            access_keys: access_keys.into_iter().flatten().collect(),
        },
        account_ids.into_iter().flatten().collect(),
    ))
}
