use std::sync::{atomic::AtomicBool, Arc};

use apca::Client;

/// Global engine state accessible to Plutonic
///
/// Data should (generally) be immutable in this object. It is useful for being
/// aware of common immutable objects such as the exchange client
pub struct EngineContext {
    pub is_running: AtomicBool,
    pub client: Client,
}

impl EngineContext {
    // TODO: Initialize from a set of settings passed
    pub fn new(client: Client) -> Arc<Self> {
        Arc::new(Self {
            is_running: AtomicBool::new(false),
            client,
        })
    }
}
