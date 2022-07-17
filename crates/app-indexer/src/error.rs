use near_jsonrpc_client::{errors::JsonRpcError, methods::health::RpcStatusError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    // #[error("{:?}", 0)]
    // LakeConfigBuild(#[from] LakeConfigBuildError),
    #[error("{:?}", _0)]
    JsonRpc(#[from] JsonRpcError<RpcStatusError>),
    #[error("{:?}", _0)]
    Reqwest(#[from] reqwest::Error),
}
