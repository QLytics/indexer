use crate::Error;
use graphql_client::GraphQLQuery;
use parking_lot::RwLock;
use qlytics_graphql::{add_blocks, add_chunks, AddBlocks, AddChunks, NewBlock, NewChunk};
use reqwest::Client;
use std::sync::Arc;

#[allow(clippy::await_holding_lock)]
pub async fn send_data(
    blocks: Arc<RwLock<Vec<NewBlock>>>,
    chunks: Arc<RwLock<Vec<NewChunk>>>,
) -> Result<(), Error> {
    let client = Client::new();

    send_blocks(&client, blocks).await?;
    send_chunks(&client, chunks).await?;

    Ok(())
}

pub async fn send_blocks(client: &Client, blocks: Arc<RwLock<Vec<NewBlock>>>) -> Result<(), Error> {
    if blocks.read().len() < 10 {
        return Ok(());
    }
    let blocks: Vec<NewBlock> = blocks.write().drain(..).collect();
    let variables = add_blocks::Variables { blocks };
    let query = AddBlocks::build_query(variables);
    client
        .post("https://api.shrm.workers.dev")
        .json(&query)
        .send()
        .await?;
    Ok(())
}

pub async fn send_chunks(client: &Client, chunks: Arc<RwLock<Vec<NewChunk>>>) -> Result<(), Error> {
    if chunks.read().len() < 10 {
        return Ok(());
    }
    let chunks: Vec<NewChunk> = chunks.write().drain(..).collect();
    let variables = add_chunks::Variables { chunks };
    let query = AddChunks::build_query(variables);
    client
        .post("https://api.shrm.workers.dev")
        .json(&query)
        .send()
        .await?;
    Ok(())
}
