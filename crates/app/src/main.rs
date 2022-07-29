use qlytics_core::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let stream = qlytics_indexer::start_indexing();
    qlytics_send::send_data(stream).await?;

    Ok(())
}
