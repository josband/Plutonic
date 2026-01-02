use std::sync::Arc;

use log::info;
use tokio::sync::mpsc;

use crate::exchange::{AlpacaExchange, LiveData};

mod context;
pub use context::*;

mod strategy;
pub use strategy::*;

mod order_executor;
pub use order_executor::*;

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

        let (order_tx, order_rx) = mpsc::channel(64);
        let mut processor = StrategyExecutor::new(self.ctx.clone(), order_tx, DummyStrategy);
        tokio::spawn(async move {
            processor.start().await;
        });

        let order_executor = OrderExecutor::new(self.ctx.clone(), order_rx);
        tokio::spawn(order_executor.start());
    }

    pub async fn shutdown(&self) {
        info!("Shutting down Plutonic");

        // Close Connection
        self.exchange.disconnect().await;
    }
}
