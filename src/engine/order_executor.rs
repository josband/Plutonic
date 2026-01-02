use std::sync::Arc;

use apca::api::v2::order::{self, Amount, Class, CreateReqInit, Side};
use num_decimal::Num;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::engine::{EngineContext, Signal, SignalType};

pub struct OrderExecutor {
    ctx: Arc<EngineContext>,
    order_rx: mpsc::Receiver<Signal>,
    cancel: CancellationToken,
}

impl OrderExecutor {
    pub fn new(ctx: Arc<EngineContext>, order_rx: mpsc::Receiver<Signal>) -> Self {
        let cancel = ctx.cancel_token().clone();

        Self {
            ctx,
            order_rx,
            cancel,
        }
    }

    pub async fn start(mut self) {
        while let Some(signal) = self.order_rx.recv().await {
            if self.cancel.is_cancelled() {
                break;
            }

            // TODO: Check if we adhere to portfolio constraints, if so place the order

            let request_params = CreateReqInit {
                class: Class::Simple,
                type_: Type::Market,
                time_in_force: order::TimeInForce::Day,
                extended_hours: false,
                ..Default::default()
            };

            let order = match signal.signal_type {
                SignalType::Buy => {
                    request_params.init(signal.symbol, Side::Buy, Amount::notional(1_000))
                }
                SignalType::Sell => {
                    request_params.init(signal.symbol, Side::Sell, Amount::notional(1_000))
                }
                _ => continue,
            };

            let order = self
                .ctx
                .client
                .issue::<order::Create>(&order)
                .await
                .unwrap();
        }
    }
}
