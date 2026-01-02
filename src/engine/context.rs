use std::sync::{atomic::AtomicBool, Arc};

use apca::Client;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::exchange::LiveData;

const LIVE_DATA_CHANNEL_SIZE: usize = 2048;

/// Global engine state accessible to Plutonic
///
/// Data should (generally) be immutable in this object. It is useful for being
/// aware of common immutable objects such as the exchange client
pub struct EngineContext {
    pub is_running: AtomicBool,
    pub client: Arc<Client>,
    cancel_token: CancellationToken,

    // TODO: Figure out a way to move these
    live_tx: broadcast::Sender<LiveData>,
    live_rx: broadcast::Receiver<LiveData>,
}

impl EngineContext {
    // TODO: Initialize from a set of settings passed
    pub fn new(client: Client) -> Arc<Self> {
        let (live_tx, live_rx) = broadcast::channel(LIVE_DATA_CHANNEL_SIZE);

        Arc::new(Self {
            is_running: AtomicBool::new(false),
            client: Arc::new(client),
            cancel_token: CancellationToken::new(),
            live_tx,
            live_rx,
        })
    }

    pub fn sender(&self) -> broadcast::Sender<LiveData> {
        self.live_tx.clone()
    }

    pub fn receiver(&self) -> broadcast::Receiver<LiveData> {
        self.live_rx.resubscribe()
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }
}
