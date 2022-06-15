use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("[Diesel] {:?}", _0)]
    Diesel(#[from] diesel::result::Error),
}
