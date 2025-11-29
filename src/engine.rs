use log::info;

use crate::exchange::AlpacaExchange;

mod context;
pub use context::*;

pub struct Plutonic {
    exchange: AlpacaExchange,
}

impl Plutonic {
    pub fn new(alpaca_exchange: AlpacaExchange) -> Self {
        Self {
            exchange: alpaca_exchange,
        }
    }

    pub async fn start(&self) {
        info!("Starting Plutonic Engine");

        // Open connection to the exchange
        self.exchange.connect().await;
    }

    pub async fn stop(&self) {
        info!("Shutting down Plutonic");

        // Close Connection
        self.exchange.disconnect().await;
    }
}
