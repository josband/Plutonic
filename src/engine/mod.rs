#![allow(unused)]

mod context;
mod risk_manager;
mod strategy;

pub use context::*;
pub use strategy::*;
use tracing::{event, Level};

use std::{collections::HashMap, sync::Arc};

use crate::{broker::data::BrokerData, engine::risk_manager::RiskManager};
use apca::{
    api::v2::{
        asset::{self, Symbol},
        assets,
        order::Order,
        position::{self, Position},
        positions,
        updates::{OrderStatus, OrderUpdate},
    },
    data::v2::stream::MarketData,
};
use tokio::sync::mpsc;

pub struct TradingEngine {
    ctx: EngineContext,
    strategy_executor: StrategyExecutor<DummyStrategy>,
    risk_manager: RiskManager,
    portfolio: HashMap<asset::Id, Position>,
}

impl TradingEngine {
    pub async fn new(ctx: EngineContext) -> Self {
        let strategy_executor = StrategyExecutor::new(DummyStrategy);
        let risk_manager = RiskManager {};

        // Get existing positions
        let portfolio = ctx
            .client
            .issue::<positions::List>(&())
            .await
            .unwrap_or_else(|_| vec![])
            .into_iter()
            .map(|p| (p.asset_id, p))
            .collect();

        Self {
            ctx,
            strategy_executor,
            risk_manager,
            portfolio,
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

    /// Handler for order updates.
    ///
    /// When submitted orders for the account are updated, the engine should
    /// be notified to manage it's internal state and take any necessary actions.
    pub async fn on_order_update(&self, order_update: OrderUpdate) {
        match order_update.event {
            OrderStatus::PartialFill | OrderStatus::Filled => {
                // This is not the fasted, but since this is not a HFT bot, the overhead is considered acceptable.
                // Maybe this will be considered
                let order = order_update.order;
                let updated_position = self
                    .ctx
                    .client
                    .issue::<position::Get>(&Symbol::Id(order.asset_id))
                    .await;

                match updated_position {
                    Ok(p) => todo!(),
                    Err(e) => event!(
                        Level::ERROR,
                        "Unable to fetch position info for {} of ID: {}",
                        order.symbol,
                        order.asset_id.0
                    ),
                }
            }
            _ => {
                event!(
                    Level::DEBUG,
                    "Received non-actionable event type {:?} for {}",
                    order_update.event,
                    order_update.order.symbol
                )
            }
        }
    }

    /// Synchronizes the internal state with the broker's data.
    ///
    /// In reality, this method should not be needed. However, it is
    /// offered as a utlity to ensure internal state is correct. If calling this method,
    /// it might be indicative of a problem with logic.
    pub async fn synchronize(&mut self) {
        let portfolio = self
            .ctx
            .client
            .issue::<positions::List>(&())
            .await
            .unwrap_or_else(|_| vec![])
            .into_iter()
            .map(|p| (p.asset_id, p))
            .collect();

        self.portfolio = portfolio;
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
