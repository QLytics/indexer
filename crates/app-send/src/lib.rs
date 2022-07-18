use futures_util::pin_mut;
use graphql_client::GraphQLQuery;
use qlytics_core::Result;
use qlytics_graphql::{add_block_data, AddBlockData, BlockData};
use reqwest::Client;
use tokio_stream::{Stream, StreamExt};

pub async fn send_data(stream: impl Stream<Item = Result<BlockData>>) -> Result<()> {
    let client = Client::new();

    pin_mut!(stream);

    let mut data = vec![];
    while let Some(block_data) = stream.next().await {
        data.push(block_data?);
        if data.len() >= 5 {
            send_block_data(&client, data.drain(..).collect()).await?;
        }
    }

    Ok(())
}

pub async fn send_block_data(client: &Client, block_data: Vec<BlockData>) -> Result<()> {
    let variables = add_block_data::Variables { block_data };
    let query = AddBlockData::build_query(variables);
    client
        .post("https://api.shrm.workers.dev")
        .json(&query)
        .send()
        .await?;
    Ok(())
}
