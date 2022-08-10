use either::Either;
use futures_util::pin_mut;
use graphql_client::GraphQLQuery;
use itertools::Itertools;
use near_lake_framework::near_indexer_primitives::types::AccountId;
use qlytics_core::Result;
use qlytics_graphql::{
    add_block_data, add_genesis_block_data, delete_accounts, AddBlockData, AddGenesisBlockData,
    BlockData, DeleteAccounts, GenesisBlockData,
};
use reqwest::Client;
use tokio_stream::{Stream, StreamExt};

pub async fn send_data(
    stream: impl Stream<Item = Result<Either<GenesisBlockData, (BlockData, Vec<AccountId>)>>>,
) -> Result<()> {
    let client = Client::new();

    pin_mut!(stream);

    let mut data = vec![];
    while let Some(block_data) = stream.next().await {
        data.push(block_data?);
        if data.len() < 10 {
            continue;
        }
        let (genesis_block_data, data): (Vec<_>, Vec<_>) =
            data.drain(..).into_iter().partition_map(|val| val);
        let (block_data, account_ids): (Vec<BlockData>, Vec<Vec<AccountId>>) =
            data.into_iter().unzip();

        send_genesis_block_data(&client, genesis_block_data).await?;

        send_block_data(&client, block_data).await?;

        let account_ids = account_ids
            .into_iter()
            .flatten()
            .map(|account_id| account_id.to_string())
            .collect();
        send_deleted_accounts(&client, account_ids).await?;
    }

    Ok(())
}

pub async fn send_block_data(client: &Client, block_data: Vec<BlockData>) -> Result<()> {
    if block_data.is_empty() {
        return Ok(());
    }
    let variables = add_block_data::Variables { block_data };
    let query = AddBlockData::build_query(variables);
    client
        .post("https://api.shrm.workers.dev")
        .json(&query)
        .send()
        .await?;
    Ok(())
}

pub async fn send_genesis_block_data(
    client: &Client,
    block_data: Vec<GenesisBlockData>,
) -> Result<()> {
    if block_data.is_empty() {
        return Ok(());
    }
    let variables = add_genesis_block_data::Variables { block_data };
    let query = AddGenesisBlockData::build_query(variables);
    client
        .post("https://api.shrm.workers.dev")
        .json(&query)
        .send()
        .await?;
    Ok(())
}

pub async fn send_deleted_accounts(client: &Client, account_ids: Vec<String>) -> Result<()> {
    if account_ids.is_empty() {
        return Ok(());
    }
    let variables = delete_accounts::Variables { account_ids };
    let query = DeleteAccounts::build_query(variables);
    client
        .post("https://api.shrm.workers.dev")
        .json(&query)
        .send()
        .await?;
    Ok(())
}
