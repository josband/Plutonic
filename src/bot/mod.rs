use std::sync::Arc;

use apca::Client;
use log::info;
use tokio::sync::mpsc;

use crate::broker::{AlpacaBroker, LiveData};
use crate::engine::{
    EngineContext, Signal, SignalType, Strategy, TradingEngine, TradingEngineHandle,
};
use crate::order_executor::OrderExecutor;

pub struct Plutonic {
    broker: AlpacaBroker,
    order_executor: OrderExecutor,
    engine_handle: TradingEngineHandle,
}

pub struct Settings;

impl Plutonic {
    #[allow(unused)]
    pub fn new(client: Client, settings: Settings) -> Self {
        // Wrap client in Arc so that it can be shared between I/O tasks and main engine task
        let client = Arc::new(client);

        // Construct main bot components
        let (data_tx, data_rx) = mpsc::unbounded_channel();
        let broker = AlpacaBroker::new(client.clone(), data_tx);

        let (order_tx, order_rx) = mpsc::channel(256);
        let order_executor = OrderExecutor::new(client.clone(), order_rx);

        let ctx = EngineContext::new(client);
        let engine_handle = TradingEngine::new(ctx, DummyStrategy, data_rx, order_tx).spawn();

        Self {
            broker,
            order_executor,
            engine_handle,
        }
    }

    pub async fn start(&self) {
        info!("Starting Plutonic...");

        self.broker.connect().await;
        self.order_executor.start().await;

        self.engine_handle.start().await;
    }

    pub async fn stop(&self) {
        info!("Stopping Plutonic...");

        self.engine_handle.stop().await;

        self.broker.disconnect().await;
        self.order_executor.stop().await;
    }
}

// *********************** REMOVE **************************************
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
// *********************************************************************
