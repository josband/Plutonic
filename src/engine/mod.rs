#![allow(unused)]

mod context;
mod portfolio;
mod strategy;

use apca::api::v2::order::Order;
pub use context::*;
use log::info;
pub use portfolio::*;
pub use strategy::*;
use tokio::sync::mpsc;

use std::sync::Arc;

use crate::broker::LiveData;

pub struct TradingEngine<S: Strategy> {
    ctx: Arc<EngineContext>,
    strategy_executor: StrategyExecutor<S>,
    portfolio: Portfolio,
}

impl<S: Strategy> TradingEngine<S> {
    pub fn new(
        ctx: EngineContext,
        strategy: S,
        data_rx: mpsc::UnboundedReceiver<LiveData>,
        order_tx: mpsc::Sender<Order>,
    ) -> Self {
        let ctx = Arc::new(ctx);
        let strategy_executor = StrategyExecutor::new(strategy);
        let portfolio = Portfolio {};

        Self {
            ctx,
            strategy_executor,
            portfolio,
        }
    }

    /// Spawns a task coordinating the trading engine.
    ///
    /// [TradingEngine] provides the interface for evaluating core trading logic. It does not
    /// spawn a task coordinating the trading engine in an event driven manner. Instead, `spawn`
    /// consumes the trading engine and spawns the coordinator task. A [TradingEngineHandle] is returned
    /// which can be used to issue commands to the engine task.
    pub fn spawn(self) -> TradingEngineHandle {
        let (command_tx, command_rx) = mpsc::channel(64);

        tokio::spawn(async move {
            // TODO: Implement trading engine coordinator task
        });

        TradingEngineHandle::new(command_tx)
    }
}

enum TradingEngineCommand {
    Start,
    Stop,
}

pub struct TradingEngineHandle {
    command_tx: mpsc::Sender<TradingEngineCommand>,
}

impl TradingEngineHandle {
    fn new(command_tx: mpsc::Sender<TradingEngineCommand>) -> Self {
        Self { command_tx }
    }

    pub async fn start(&self) {
        let _ = self.command_tx.send(TradingEngineCommand::Start).await;
    }

    pub async fn stop(&self) {
        let _ = self.command_tx.send(TradingEngineCommand::Stop).await;
    }
}
