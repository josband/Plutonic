#![allow(unused)]

mod context;
mod portfolio;
mod risk_manager;
mod strategy;

pub use context::*;
pub use portfolio::*;
pub use strategy::*;

use std::sync::Arc;

use crate::{broker::data::BrokerData, engine::risk_manager::RiskManager};
use apca::{
    api::v2::{order::Order, updates::OrderUpdate},
    data::v2::stream::MarketData,
};
use log::info;
use tokio::sync::mpsc;

pub struct TradingEngine {
    ctx: Arc<EngineContext>,
    strategy_executor: StrategyExecutor<DummyStrategy>,
    risk_manager: RiskManager,
    portfolio: Portfolio,
}

impl TradingEngine {
    pub fn new(ctx: EngineContext) -> Self {
        let ctx = Arc::new(ctx);
        let strategy_executor = StrategyExecutor::new(DummyStrategy);
        let portfolio = Portfolio {};
        let risk_manager = RiskManager {};

        Self {
            ctx,
            strategy_executor,
            risk_manager,
            portfolio,
        }
    }

    pub async fn on_market_data(&self, data: BrokerData) -> Option<Order> {
        let signal = self.strategy_executor.evaluate_strategies(data).await;
        if signal.signal_type == SignalType::Neutral {
            return None;
        }

        todo!("Need to evaluate risk against the portfolio")

        // Check if the signal adheres to risk strategies
        //
        // If passes, generate an order adhering to risk strategies
    }

    pub async fn on_order_update(&self, order_update: OrderUpdate) {
        todo!("Must update internal state with order details")

        // Update portfolio with new order details
    }

    pub async fn synchronize(&self) {
        // Synchronize portfolio with broker data. Ensures portfolio data matches the account
    }
}

// *********************** REMOVE **************************************
struct DummyStrategy;

impl Strategy for DummyStrategy {
    fn process(&self, _data: &BrokerData) -> Signal {
        dbg!("Process called");
        Signal {
            signal_type: SignalType::Neutral,
            symbol: String::new(),
        }
    }
}
// *********************************************************************
