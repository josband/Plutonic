use std::{collections::HashMap, sync::Arc};

use apca::{
    api::v2::{
        account::{self, Account},
        asset::Symbol,
        order::{self, Amount, Order},
        orders::{self, ListReq},
        position::{self, Position},
        positions,
        updates::{OrderStatus, OrderUpdate},
    },
    data::v2::{stream::Bar, trades},
    Client,
};
use chrono::Utc;
use num_decimal::Num;
use tracing::{event, Level};

/// Manages state relating to an account
pub struct PortfolioManager {
    client: Arc<Client>,
    account: Account,
    positions: HashMap<String, Position>,
    open_orders: HashMap<String, HashMap<order::Id, Order>>,
}

impl PortfolioManager {
    pub async fn new(client: Arc<Client>) -> Self {
        // Get existing positions
        let positions = client
            .issue::<positions::List>(&())
            .await
            .unwrap_or_else(|_| vec![])
            .into_iter()
            .map(|p| (p.symbol.clone(), p))
            .collect();

        let open_orders = client
            .issue::<orders::List>(&ListReq {
                status: orders::Status::Open,
                ..Default::default()
            })
            .await
            .unwrap_or_else(|_| vec![])
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc: HashMap<String, HashMap<order::Id, Order>>, o| {
                    acc.entry(o.symbol.clone()).or_default().insert(o.id, o);
                    acc
                },
            );

        // Get account information
        let account = client.issue::<account::Get>(&()).await.unwrap();

        Self {
            client,
            account,
            positions,
            open_orders,
        }
    }

    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    pub fn cash_available(&self) -> Num {
        self.account.cash.clone()
    }

    pub fn equity(&self) -> Num {
        self.account.equity.clone()
    }

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
                        self.positions.insert(p.symbol.clone(), p);
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

    pub async fn on_bar(&mut self, bar: &Bar) {
        // TODO: Optimize. These values can be calculated via the bars.
        let position = self
            .client
            .issue::<position::Get>(&Symbol::Sym(bar.symbol.clone()))
            .await
            .unwrap();
        let account = self.client.issue::<account::Get>(&()).await.unwrap();

        self.positions.insert(position.symbol.clone(), position);
        self.account = account;
    }

    pub async fn dedicated_money(&self, symbol: &str) -> f64 {
        let position_value = self
            .positions
            .get(symbol)
            .map(|p| {
                p.market_value
                    .as_ref()
                    .map(|v| v.to_f64().expect("Cannot convert position to f64"))
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0);

        let order_values = self
            .open_orders
            .get(symbol)
            .map(|orders| orders.values())
            .unwrap_or_default();

        let mut order_value = 0.0;
        let mut current_price = None;
        for o in order_values {
            order_value = match o.amount {
                Amount::Quantity { ref quantity } => {
                    if current_price.is_none() {
                        let now = Utc::now();
                        current_price = Some(
                            self.client
                                .issue::<trades::List>(
                                    &trades::ListReqInit {
                                        limit: Some(1),
                                        ..Default::default()
                                    }
                                    .init(
                                        o.symbol.clone(),
                                        now,
                                        now,
                                    ),
                                )
                                .await
                                .unwrap()
                                .trades[0]
                                .price
                                .to_f64()
                                .unwrap_or_default(),
                        );
                    }

                    quantity.to_f64().unwrap_or(0.0)
                        * current_price.as_ref().copied().unwrap_or(0.0)
                }
                Amount::Notional { ref notional } => notional.to_f64().unwrap_or(0.0),
            };
        }

        position_value + order_value
    }
}
