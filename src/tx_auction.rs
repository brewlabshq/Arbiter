use std::{
    sync::{Arc, mpsc},
    thread::JoinHandle,
};

use crossbeam_queue::ArrayQueue;

#[derive(Clone, Debug)]
pub struct InternalTransaction {
    pub id: u64,
    pub txs: Vec<String>,
    pub tip_lamports: u64,
    pub timestamp: u128,
}

pub struct TxAuction {
    pub pending_tx: Arc<ArrayQueue<InternalTransaction>>,
    pub time_delay: u64,
    pub auction_tx: (
        mpsc::Sender<InternalTransaction>,
        mpsc::Receiver<InternalTransaction>,
    ),
}

impl TxAuction {
    pub fn new(pending_tx: Arc<ArrayQueue<InternalTransaction>>, time_delay_ms: u64) -> Self {
        Self {
            pending_tx,
            time_delay: time_delay_ms,
            auction_tx: mpsc::channel(),
        }
    }
}
