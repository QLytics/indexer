use crate::Result;
use either::Either;
use itertools::Itertools;
use near_primitives::state_record::StateRecord;
use qlytics_graphql::{AccessKey, Account};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Records {
    genesis_height: u64,
    records: Vec<StateRecord>,
}

pub(crate) async fn handle_genesis() -> Result<(Vec<Account>, Vec<AccessKey>)> {
    let Records { genesis_height, records } = Client::new().get("https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/mainnet/genesis.json").send().await?.json().await?;
    let (accounts, access_keys): (Vec<Account>, Vec<AccessKey>) = records
        .into_iter()
        .filter_map(|record| match record {
            StateRecord::Account { account_id, .. } => Some(Either::Left(Account::new(
                &account_id,
                None,
                genesis_height,
            ))),
            StateRecord::AccessKey {
                public_key,
                account_id,
                access_key,
            } => Some(Either::Right(AccessKey::new(
                &public_key,
                &account_id,
                &access_key.permission,
                None,
                genesis_height,
            ))),
            _ => None,
        })
        .partition_map(|val| val);
    Ok((accounts, access_keys))
}
