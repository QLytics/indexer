use parking_lot::RwLock;
use qlytics_db::Database;
use std::{env, sync::Arc};
use thiserror::Error;

#[tokio::main]
pub async fn main() -> Result<(), AppError> {
    dotenv::dotenv().ok();
    let database = Arc::new(RwLock::new(Database::new(
        &env::var("DATABASE_URL").unwrap(),
    )));
    qlytics_indexer::start_indexing(database).await?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("[Indexer]: {}", _0)]
    Indexer(#[from] qlytics_indexer::Error),
}
