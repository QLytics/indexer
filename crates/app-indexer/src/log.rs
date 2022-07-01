use crate::Error;
use chrono::{DateTime, Utc};
use near_jsonrpc_client::{methods, JsonRpcClient};
use parking_lot::RwLock;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

#[allow(clippy::await_holding_lock)]
pub(crate) async fn log(
    block_height: u64,
    client: &Arc<JsonRpcClient>,
    time: &Arc<RwLock<Instant>>,
    eta: &Arc<RwLock<VecDeque<(Duration, u64)>>>,
    misses: &Arc<RwLock<u32>>,
) -> Result<(), Error> {
    let mut time = time.write();
    let elapsed = time.elapsed();
    if elapsed > Duration::from_secs(10) {
        *time = Instant::now();
        drop(time);
        let current_block_height = get_current_block_height(client).await?;
        let mut eta = eta.write();
        eta.push_back((elapsed, block_height));
        if eta.len() > 1 {
            if eta.len() > 5 {
                eta.pop_front();
            }

            let (_, first_block_height) = eta.front().unwrap();
            let (_, last_block_height) = eta.back().unwrap();
            let total_dur: Duration = eta.iter().map(|(t, _)| t).sum();
            let total_blocks = last_block_height - first_block_height;
            let blocks_per_millis = total_blocks as f64 / total_dur.as_millis() as f64;
            let utc: DateTime<Utc> = Utc::now();

            let eta = (current_block_height - block_height) as f64 / blocks_per_millis;
            let eta = Duration::from_millis(eta as u64);

            println!(
                "[{}] Height: {}, BPS: {:.1}, Misses: {}, ETA: {}",
                utc.format("%Y-%m-%d %H:%M:%S"),
                block_height,
                blocks_per_millis as f32 * 1_000.,
                misses.read(),
                humantime::Duration::from(eta)
            );
        }
    }
    Ok(())
}

async fn get_current_block_height(client: &Arc<JsonRpcClient>) -> Result<u64, Error> {
    let status = client.call(methods::status::RpcStatusRequest).await?;

    Ok(status.sync_info.latest_block_height)
}
