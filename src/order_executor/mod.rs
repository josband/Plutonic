use std::sync::Arc;

use apca::{
    api::v2::{
        order::{self as apca_order, CreateError, Order},
        updates::{OrderUpdate, OrderUpdates},
    },
    Client, RequestError, Subscribable,
};
use futures::StreamExt;
use log::info;

type OrderUpdatesStream = <OrderUpdates as Subscribable>::Stream;

#[allow(unused)]
pub struct OrderExecutor {
    client: Arc<Client>,
    stream: OrderUpdatesStream,
    // Add something to track active orders
}

impl OrderExecutor {
    pub async fn new(client: Arc<Client>) -> Self {
        info!("Opening live order update connection");
        let (stream, _) = client.subscribe::<OrderUpdates>().await.unwrap();

        Self { client, stream }
    }

    /// Await the next order update event from the broker
    pub async fn next_order_update(&mut self) -> Option<OrderUpdate> {
        self.stream
            .next()
            .await
            .map(|update| update.unwrap().unwrap())
    }

    /// Submits an order to the broker for execution
    #[allow(unused)]
    pub async fn submit_order(&self, order: Order) -> Result<(), RequestError<CreateError>> {
        info!(
            "Submitting order {} for {}",
            order.id.as_hyphenated(),
            order.symbol
        );
        let request = apca_order::CreateReqInit {
            ..Default::default()
        }
        .init(order.symbol, order.side, order.amount);

        let res = self.client.issue::<apca_order::Create>(&request).await;

        res.map(|_| ())
    }
}
