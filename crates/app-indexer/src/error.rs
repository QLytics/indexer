use actix_web::rt::task::JoinError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{:?}", 0)]
    JoinError(#[from] JoinError),
}
