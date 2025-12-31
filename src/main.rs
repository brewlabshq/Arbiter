use std::sync::Arc;

use crossbeam_queue::ArrayQueue;

use crate::{api::create_router, tx_auction::InternalTransaction};

mod api;
mod tx_auction;

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Arbiter");

    let tx_queue = Arc::new(ArrayQueue::<InternalTransaction>::new(1 << 16));

    // Async api server
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(create_router(tx_queue));

    Ok(())
}
