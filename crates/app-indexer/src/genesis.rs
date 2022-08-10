use crate::Result;
use either::Either;
use itertools::Itertools;
use near_primitives::state_record::StateRecord;
use qlytics_graphql::Account;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Records {
    genesis_height: u64,
    records: Vec<StateRecord>,
}

pub(crate) async fn handle_genesis() -> Result<Vec<Account>> {
    let Records { genesis_height, records } = Client::new().get("https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/mainnet/genesis.json").send().await?.json().await?;
    let (accounts, _access_keys): (Vec<Account>, Vec<()>) = records
        .into_iter()
        .filter_map(|record| {
            match record {
                StateRecord::Account { account_id, .. } => Some(Either::Left(Account::new(
                    &account_id,
                    None,
                    genesis_height,
                ))),
                StateRecord::AccessKey { .. } => {
                    // TODO
                    Some(Either::Right(()))
                }
                _ => None,
            }
        })
        .partition_map(|val| val);
    Ok(accounts)
}
