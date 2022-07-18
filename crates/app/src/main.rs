use parking_lot::RwLock;
use qlytics_core::Result;
use qlytics_db::Database;
use std::{env, sync::Arc};

#[tokio::main]
pub async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let database = Arc::new(RwLock::new(Database::new(
        &env::var("DATABASE_URL").unwrap(),
    )));
    let stream = qlytics_indexer::start_indexing(database);
    qlytics_send::send_data(stream).await?;

    Ok(())
}
