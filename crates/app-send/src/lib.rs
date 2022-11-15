use async_stream::try_stream;
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

pub async fn prepare_data(
    stream: impl Stream<Item = Result<Either<GenesisBlockData, (BlockData, Vec<AccountId>)>>>,
) -> impl Stream<Item = Result<(Vec<GenesisBlockData>, Vec<BlockData>, Vec<String>)>> {
    try_stream! {
        let mut data = vec![];
        for await block_data in stream {
            data.push(block_data?);
            if data.len() < 100 {
                continue;
            }
            let (genesis_block_data, data): (Vec<_>, Vec<_>) =
                data.drain(..).into_iter().partition_map(|val| val);
            let (block_data, account_ids): (Vec<BlockData>, Vec<Vec<AccountId>>) =
                data.into_iter().unzip();
            let account_ids = account_ids
                .into_iter()
                .flatten()
                .map(|account_id| account_id.to_string())
                .collect();

            yield (genesis_block_data, block_data, account_ids);
        }
    }
}

pub async fn send_data(
    stream: impl Stream<Item = Result<(Vec<GenesisBlockData>, Vec<BlockData>, Vec<String>)>>,
) {
    pin_mut!(stream);

    while let Some(data) = stream.next().await {
        let (genesis_block_data, block_data, account_ids) = data.unwrap();
        let client = Client::new();
        send_genesis_block_data(&client, genesis_block_data)
            .await
            .unwrap();
        send_block_data(&client, block_data).await.unwrap();
        send_deleted_accounts(&client, account_ids).await.unwrap();
    }
}

pub async fn send_block_data(client: &Client, block_data: Vec<BlockData>) -> Result<()> {
    if block_data.is_empty() {
        return Ok(());
    }
    let variables = add_block_data::Variables { block_data };
    let query = AddBlockData::build_query(variables);

    #[cfg(not(debug_assertions))]
    client
        .post("https://api.shrm.workers.dev")
        .json(&query)
        .send()
        .await?;
    #[cfg(debug_assertions)]
    let res = client
        .post("https://api.shrm.workers.dev")
        .json(&query)
        .send()
        .await?;
    #[cfg(debug_assertions)]
    if !res.status().is_success() {
        let text = res.text().await?;
        dbg!(text);
    }
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
