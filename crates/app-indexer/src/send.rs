use crate::Error;
use graphql_client::GraphQLQuery;
use parking_lot::RwLock;
use qlytics_graphql::{
    add_blocks::{self, NewBlock},
    AddBlocks,
};
use reqwest::Client;
use std::sync::Arc;

#[allow(clippy::await_holding_lock)]
pub async fn send_data(blocks: Arc<RwLock<Vec<NewBlock>>>) -> Result<(), Error> {
    let client = Client::new();

    send_blocks(&client, blocks).await?;

    Ok(())
}

pub async fn send_blocks(client: &Client, blocks: Arc<RwLock<Vec<NewBlock>>>) -> Result<(), Error> {
    if blocks.read().len() >= 10 {
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
