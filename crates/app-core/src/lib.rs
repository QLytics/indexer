use near_jsonrpc_client::{errors::JsonRpcError, methods::health::RpcStatusError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{:?}", _0)]
    JsonRpc(#[from] JsonRpcError<RpcStatusError>),
    #[error("{:?}", _0)]
    Reqwest(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
