mod risk_manager;
mod strategy;

pub use strategy::*;

use num_decimal::Num;
use std::{cmp::min, collections::HashMap, sync::Arc};
use tracing::{event, Level};

use crate::engine::{
    risk_manager::{RiskDecision, RiskManager},
    signal::{Signal, SignalDirection},
};
use apca::{
    api::v2::{
        account::{self, Account},
        asset::Symbol,
        order::{self, Amount, Class, CreateReq, Side, TimeInForce, Type},
        position::{self, Position},
        positions,
        updates::{OrderStatus, OrderUpdate},
    },
    data::v2::stream::Bar,
    Client,
};

/// Engine containing core strategy logic, order management, and risk management.
///
/// The TradingEngine is the sole source of truth of positions, order status, cash balance etc.
pub struct TradingEngine {
    client: Arc<Client>,
    strategy_executor: StrategyExecutor<DummyStrategy>,
    risk_manager: RiskManager,
    portfolio: HashMap<String, Position>,
    account: Account,
}

impl TradingEngine {
    pub async fn new(client: Arc<Client>) -> Self {
        let strategy_executor = StrategyExecutor::new(DummyStrategy);
        let risk_manager = RiskManager::new();

        // Get existing positions
        let portfolio = client
            .issue::<positions::List>(&())
            .await
            .unwrap_or_else(|_| vec![])
            .into_iter()
            .map(|p| (p.symbol.clone(), p))
            .collect();

        // Get account information
        let account = client.issue::<account::Get>(&()).await.unwrap();

        Self {
            client,
            strategy_executor,
            risk_manager,
            portfolio,
            account,
        }
    }

    pub async fn on_bar(&self, bar: Bar) -> Option<CreateReq> {
        event!(
            Level::INFO,
            "Evaluating market strategies on market update for {}",
            bar.symbol
        );

        let signal = self.strategy_executor.evaluate_strategies(&bar).await;
        if signal.direction() == SignalDirection::Neutral {
            event!(Level::INFO, "Neutral signal generated. Taking no action");
            return None;
        }

        let initial_order = self.create_order_req(signal.direction().order_side()?, &bar)?;

        match self.risk_manager.evaluate_order(&initial_order) {
            RiskDecision::Accept => Some(initial_order),
            RiskDecision::Modify(new) => Some(*new),
            RiskDecision::Reject(msg) => {
                event!(Level::WARN, "Order rejected: {}", msg);
                None
            }
        }
    }

    fn create_order_req(&self, side: Side, bar: &Bar) -> Option<CreateReq> {
        let order_amount = match side {
            Side::Buy => Amount::notional(min(
                &self.account.equity * Num::new(2, 100),
                self.account.cash.clone(),
            )),
            Side::Sell => {
                let current_position = self.portfolio.get(&bar.symbol)?;
                let position_size = current_position.quantity_available.clone()
                    * current_position.current_price.as_ref()?;

                Amount::notional(min(&self.account.equity * Num::new(2, 100), position_size))
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

    /// Handler for order updates.
    ///
    /// When submitted orders for the account are updated, the engine should
    /// be notified to manage it's internal state and take any necessary actions.
    pub async fn on_order_update(&mut self, order_update: OrderUpdate) {
        match order_update.event {
            OrderStatus::PartialFill | OrderStatus::Filled => {
                // This is not the fastest, but since this is not a HFT bot, the overhead is considered acceptable. but it should be improved
                let order = order_update.order;
                let updated_position = self
                    .client
                    .issue::<position::Get>(&Symbol::Id(order.asset_id))
                    .await;

                match updated_position {
                    Ok(p) => {
                        self.portfolio.insert(p.symbol.clone(), p);
                    }
                    Err(e) => event!(
                        Level::ERROR,
                        "Unable to fetch position info for {} of ID: {}, {}",
                        order.symbol,
                        order.asset_id.0,
                        e
                    ),
                }

                self.account = self.client.issue::<account::Get>(&()).await.unwrap();
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
}

// *********************** REMOVE **************************************
struct DummyStrategy;

impl Strategy for DummyStrategy {
    fn process(&self, data: &Bar) -> Signal {
        event!(Level::DEBUG, "Process called");

        Signal::new(data.symbol.to_string(), SignalDirection::Buy)
    }
}
// *********************************************************************
