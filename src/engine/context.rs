use std::sync::{atomic::AtomicBool, Arc};

use apca::Client;
use tokio::sync::broadcast;

use crate::exchange::LiveData;

const LIVE_DATA_CHANNEL_SIZE: usize = 2048;

/// Global engine state accessible to Plutonic
///
/// Data should (generally) be immutable in this object. It is useful for being
/// aware of common immutable objects such as the exchange client
pub struct EngineContext {
    pub is_running: AtomicBool,
    pub client: Client,
    live_tx: broadcast::Sender<LiveData>,
    live_rx: broadcast::Receiver<LiveData>,
}

impl EngineContext {
    // TODO: Initialize from a set of settings passed
    pub fn new(client: Client) -> Arc<Self> {
        let (tx, rx) = broadcast::channel(LIVE_DATA_CHANNEL_SIZE);

        Arc::new(Self {
            is_running: AtomicBool::new(false),
            client,
            live_tx: tx,
            live_rx: rx,
        })
    }

    pub fn sender(&self) -> broadcast::Sender<LiveData> {
        self.live_tx.clone()
    }

    pub fn receiver(&self) -> broadcast::Receiver<LiveData> {
        self.live_rx.resubscribe()
    }
}
