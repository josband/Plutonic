mod portfolio_manager;
mod risk_manager;
mod strategy;

pub use strategy::*;

use num_decimal::Num;
use std::{cmp::min, sync::Arc};
use tracing::{event, Level};

use crate::engine::{
    portfolio_manager::PortfolioManager,
    risk_manager::{RiskDecision, RiskManager},
    signal::{Signal, SignalDirection},
};
use apca::{
    api::v2::{
        order::{self, Amount, Class, CreateReq, Side, TimeInForce, Type},
        updates::OrderUpdate,
    },
    data::v2::stream::Bar,
    Client,
};

/// Engine containing core strategy logic, order management, and risk management.
///
/// The TradingEngine is the sole source of truth of positions, order status, cash balance etc.
pub struct TradingEngine {
    // client: Arc<Client>,
    strategy_executor: StrategyExecutor,
    risk_manager: RiskManager,
    portfolio_manager: PortfolioManager,
}

impl TradingEngine {
    pub async fn new(client: Arc<Client>) -> Self {
        let risk_manager = RiskManager::new();
        let portfolio_manager = PortfolioManager::new(client.clone()).await;
        let strategy_executor = StrategyExecutor::new();

        Self {
            // client,
            strategy_executor,
            risk_manager,
            portfolio_manager,
        }
    }

    pub fn add_strategy<S>(&mut self, strategy: S)
    where
        S: Strategy,
    {
        self.strategy_executor.register_strategy(strategy);
    }

    pub async fn on_bar(&mut self, bar: Bar) -> Option<CreateReq> {
        event!(
            Level::INFO,
            "Evaluating market strategies on market update for {}",
            bar.symbol
        );
        self.portfolio_manager.on_bar(&bar).await;

        let signal: Signal = self.strategy_executor.evaluate_strategies(&bar).await;
        if signal.direction() == SignalDirection::Neutral {
            event!(Level::INFO, "Neutral signal generated. Taking no action");
            return None;
        }

        let initial_order = self.create_order_req(signal.direction().order_side()?, &bar)?;

        match self
            .risk_manager
            .evaluate_order(&initial_order, &self.portfolio_manager)
            .await
        {
            RiskDecision::Accept => Some(initial_order),
            RiskDecision::Modify(new) => Some(*new),
            RiskDecision::Reject => {
                event!(Level::WARN, "Order rejected for {}", &bar.symbol);
                None
            }
        }
    }

    /// Handler for order updates.
    ///
    /// When submitted orders for the account are updated, the engine should
    /// be notified to manage it's internal state and take any necessary actions.
    pub async fn on_order_update(&mut self, order_update: OrderUpdate) {
        self.portfolio_manager.on_order_update(order_update).await;
    }

    fn create_order_req(&self, side: Side, bar: &Bar) -> Option<CreateReq> {
        let order_amount = match side {
            Side::Buy => Amount::notional(min(
                &self.portfolio_manager.equity() * Num::new(2, 100),
                self.portfolio_manager.cash_available(),
            )),
            Side::Sell => {
                let current_position = self.portfolio_manager.get_position(&bar.symbol)?;
                let position_size = current_position.quantity_available.clone()
                    * current_position.current_price.as_ref()?;

                Amount::notional(min(
                    &self.portfolio_manager.equity() * Num::new(2, 100),
                    position_size,
                ))
            }
        };

        Some(
            order::CreateReqInit {
                class: Class::Simple,
                type_: Type::Limit,
                limit_price: Some(bar.close_price.clone()),
                time_in_force: TimeInForce::Day,
                ..Default::default()
            }
            .init(&bar.symbol, side, order_amount),
        )
    }
}
