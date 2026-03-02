#![allow(unused)]

use std::str::FromStr;

use apca::api::v2::{
    asset::Symbol,
    order::{Amount, CreateReq, Side},
};
use num_decimal::Num;
use tracing::{event, Level};

use crate::engine::portfolio_manager::PortfolioManager;

#[derive(Debug, PartialEq, Eq)]
pub enum RiskDecision {
    Accept,
    Reject,
    Modify(Box<CreateReq>),
}

pub struct RiskManager {
    max_trade_size: f64,
    max_total_position_size: f64,
}

impl RiskManager {
    pub fn new() -> Self {
        Self {
            max_trade_size: 0.01,
            max_total_position_size: 0.1,
        }
    }

    pub async fn evaluate_order(
        &self,
        order: &CreateReq,
        portfolio: &PortfolioManager,
    ) -> RiskDecision {
        let order_symbol = match &order.symbol {
            Symbol::Sym(s) => s,
            _ => {
                event!(Level::ERROR, "Passed in order request is not a Symbol::Sym");
                panic!("Falied to read order symbol.");
            }
        };

        let mut order_value = match &order.amount {
            Amount::Quantity { quantity } => {
                quantity.to_f64().unwrap()
                    * order
                        .limit_price
                        .clone()
                        .expect("Using non-limit order")
                        .to_f64()
                        .unwrap()
            }
            Amount::Notional { notional } => notional.to_f64().unwrap(),
        };

        let mut order_changed = false;
        let money_committed = portfolio.dedicated_money(order_symbol).await;
        let available_funds = portfolio.cash_available().to_f64().unwrap();
        let equity = portfolio.equity().to_f64().unwrap();

        // Ensure trade is feasible
        if (order.side == Side::Buy && available_funds == 0.0)
            || (order.side == Side::Sell && money_committed == 0.0)
        {
            return RiskDecision::Reject;
        }

        let mut current_order = order.clone();
        if order_value > available_funds {
            order_value = available_funds;
            current_order.amount = Amount::notional(to_num(order_value));
            order_changed = true;
        }

        if order_value + money_committed > equity {
            order_value = equity - money_committed;
            current_order.amount = Amount::notional(to_num(order_value));
            order_changed = true;
        }

        if order_value / equity > self.max_trade_size {
            order_value = self.max_trade_size * equity;
            current_order.amount = Amount::notional(to_num(order_value));
            order_changed = true;
        }

        if order_changed {
            RiskDecision::Modify(Box::new(current_order))
        } else {
            RiskDecision::Accept
        }
    }
}

fn to_num(value: f64) -> Num {
    FromStr::from_str(&value.to_string()).unwrap()
}
