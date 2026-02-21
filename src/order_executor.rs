use std::{collections::HashMap, sync::Arc};

use apca::{
    api::v2::{
        order::{self as apca_order, CreateError, Id, Order},
        orders::{self, ListReq},
        updates::{OrderStatus, OrderUpdate, OrderUpdates},
    },
    Client, RequestError, Subscribable,
};
use futures::StreamExt;
use tracing::{event, Level};

type OrderUpdateStream = <OrderUpdates as Subscribable>::Stream;

#[allow(unused)]
pub struct OrderExecutor {
    client: Arc<Client>,
    stream: OrderUpdateStream,
    active_orders: HashMap<Id, Order>,
}

impl OrderExecutor {
    pub async fn new(client: Arc<Client>) -> Self {
        event!(Level::INFO, "Establishing live order update connection");
        let (stream, _) = client.subscribe::<OrderUpdates>().await.unwrap();

        let orders = client
            .issue::<orders::List>(&ListReq {
                status: orders::Status::Open,
                ..Default::default()
            })
            .await
            .unwrap_or_else(|_| vec![])
            .into_iter()
            .map(|o| (o.id, o))
            .collect();

        Self {
            client,
            stream,
            active_orders: orders,
        }
    }

    /// Await the next order update event from the broker.
    ///
    /// The OrderExecutor internally tracks order statuses for bookkeeping. Events
    /// should be passed to both the executor and the [TradingEngine].
    pub async fn next_order_update(&mut self) -> Option<OrderUpdate> {
        self.stream
            .next()
            .await
            .map(|update| update.unwrap().unwrap())
    }

    pub async fn on_order_update(&mut self, update: OrderUpdate) {
        match update.event {
            OrderStatus::New | OrderStatus::Replaced => {
                self.active_orders.insert(update.order.id, update.order);
            }
            OrderStatus::Filled | OrderStatus::Canceled | OrderStatus::Expired => {
                if self.active_orders.remove(&update.order.id).is_none() {
                    event!(
                        Level::WARN,
                        "Order {} was cancelled but not found among active orders",
                        update.order.id.as_hyphenated()
                    )
                };
            }
            OrderStatus::PartialFill => {
                // TODO: Separate branch for testing purposes. Can be moved to New/Replaced if confirmed
                self.active_orders.insert(update.order.id, update.order);
            }
            _ => {
                event!(
                    Level::INFO,
                    "Received {:?} event for {}",
                    update.event,
                    update.order.symbol
                )
            }
        };
    }

    /// Submits an order to the broker for execution
    pub async fn submit_order(&self, order: Order) -> Result<(), RequestError<CreateError>> {
        event!(
            Level::INFO,
            "Submitting order {} for {}",
            order.id.as_hyphenated(),
            order.symbol
        );

        let request = apca_order::CreateReqInit {
            ..Default::default()
        }
        .init(order.symbol, order.side, order.amount);

        self.client
            .issue::<apca_order::Create>(&request)
            .await
            .map(|_| ())
    }
}
