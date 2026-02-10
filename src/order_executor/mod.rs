use std::sync::Arc;

use apca::{
    api::v2::order::{self, Class, CreateReqInit, Order, TimeInForce, Type},
    Client,
};
use log::info;
use tokio::sync::mpsc;

/// Service responsible for executing and managing orders.
///
/// `OrderExecutor` acts as an abstration between the trading engine logic
/// and the 3rd party broker responsible for trade execution. Orders will be submitted,
/// and on various order events, will track progress and inform the engine of any changes
/// to the portfolio state.
pub struct OrderExecutor {
    start_tx: mpsc::Sender<()>,
    stop_tx: mpsc::Sender<()>,
}

impl OrderExecutor {
    pub fn new(client: Arc<Client>, mut order_rx: mpsc::Receiver<Order>) -> Self {
        // TODO: Create struct to consolidate state and start
        let (stop_tx, mut stop_rx) = mpsc::channel(8);
        let (start_tx, mut start_rx) = mpsc::channel(8);
        let alpaca_client = client.clone();
        tokio::spawn(async move {
            'a: while let Some(()) = start_rx.recv().await {
                info!("OrderExecutor started.");
                loop {
                    tokio::select! {
                        stop_signal = stop_rx.recv() => {
                            if stop_signal.is_none() {
                                break 'a;
                            }

                            info!("Stopping OrderExecutor.");
                            break;
                        },
                        order_option = order_rx.recv() => {
                            if order_option.is_none() {
                                break 'a;
                            }

                            info!("Processing order request.");
                            let order = order_option.unwrap();
                            let request_params = CreateReqInit {
                                class: Class::Simple,
                                type_: Type::Market,
                                time_in_force: TimeInForce::Day,
                                extended_hours: false,
                                ..Default::default()
                            };

                            let order_req = request_params.init(order.symbol, order.side, order.amount);

                            let _order = alpaca_client
                                .issue::<order::Create>(&order_req)
                                .await
                                .unwrap();
                        }
                    }
                }
            }

            info!("Shutting Down OrderExecutor.");
        });

        Self { start_tx, stop_tx }
    }

    pub async fn start(&self) {
        let _ = self.start_tx.send(()).await;
    }

    pub async fn stop(&self) {
        let _ = self.stop_tx.send(()).await;
    }
}
