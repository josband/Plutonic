use log::info;

use crate::exchange::AlpacaExchange;

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
}
