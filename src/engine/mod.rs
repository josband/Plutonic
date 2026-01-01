use std::sync::Arc;

use log::info;

use crate::exchange::{AlpacaExchange, LiveData};

mod context;
pub use context::*;

mod strategy;
pub use strategy::*;

pub struct Plutonic {
    ctx: Arc<EngineContext>,
    exchange: AlpacaExchange,
}

struct DummyStrategy;

impl Strategy for DummyStrategy {
    fn process(&self, _data: &LiveData) -> Signal {
        dbg!("Process called");
        Signal {
            signal_type: SignalType::Hold,
            symbol: String::new(),
        }
    }
}

impl Plutonic {
    pub fn new(ctx: Arc<EngineContext>) -> Self {
        let exchange = AlpacaExchange::new(ctx.clone());

        Self { ctx, exchange }
    }

    pub async fn start(&self) {
        info!("Starting Plutonic Engine");

        // Open connection to the exchange
        self.exchange.connect().await;
        let mut processor = StrategyExecutor::new(self.ctx.clone(), DummyStrategy);

        tokio::spawn(async move {
            processor.start().await;
        });
    }

    pub async fn stop(&self) {
        info!("Shutting down Plutonic");

        // Close Connection
        self.exchange.disconnect().await;
    }
}
