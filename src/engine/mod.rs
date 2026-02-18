#![allow(unused)]

mod context;
mod risk_manager;
mod strategy;

pub use context::*;
pub use strategy::*;
use tracing::{event, Level};

use std::sync::Arc;

use crate::{broker::data::BrokerData, engine::risk_manager::RiskManager};
use apca::{
    api::v2::{order::Order, updates::OrderUpdate},
    data::v2::stream::MarketData,
};
use tokio::sync::mpsc;

pub struct TradingEngine {
    ctx: EngineContext,
    strategy_executor: StrategyExecutor<DummyStrategy>,
    risk_manager: RiskManager,
}

impl TradingEngine {
    pub fn new(ctx: EngineContext) -> Self {
        let strategy_executor = StrategyExecutor::new(DummyStrategy);
        let risk_manager = RiskManager {};

        Self {
            ctx,
            strategy_executor,
            risk_manager,
        }
    }

    pub async fn on_market_data(&self, data: BrokerData) -> Option<Order> {
        event!(
            Level::INFO,
            "Evaluating market strategies on market update for {}",
            data.symbol()
        );

        let signal = self.strategy_executor.evaluate_strategies(data).await;
        if signal.direction() == SignalDirection::Neutral {
            event!(Level::INFO, "Neutral signal generated. Taking no action");
            return None;
        }

        todo!("Need to evaluate risk against the portfolio")

        // Generate an order, then check against risk model and if it passes, return the order
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
    fn process(&self, data: &BrokerData) -> Signal {
        event!(Level::DEBUG, "Process called");

        Signal::new(data.symbol().to_string(), SignalDirection::Buy)
    }
}
// *********************************************************************
