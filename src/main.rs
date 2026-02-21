use std::sync::Arc;

use apca::{ApiInfo, Client};
use plutonic::{
    broker::AlpacaBroker,
    engine::{EngineContext, TradingEngine},
    order_executor::OrderExecutor,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{event, Level};
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
#[allow(unused)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_environment();

    event!(Level::INFO, "Building Alpaca API Information");
    let api_info = ApiInfo::from_env().inspect_err(|_| {
        event!(
            Level::ERROR,
            "Environment is not properly set. Make sure to set both the API key and secret"
        );
    })?;

    let client = Arc::new(Client::new(api_info));
    event!(
        Level::DEBUG,
        "Api key: {} Secret: {}",
        client.api_info().key_id,
        client.api_info().secret
    );

    event!(Level::INFO, "Starting Plutonic");

    let cancel = CancellationToken::new();

    // Start broker task
    let (data_tx, mut data_rx) = mpsc::unbounded_channel();
    let broker_cancel = cancel.clone();
    let broker_client = client.clone();
    tokio::spawn(async move {
        let mut broker = AlpacaBroker::connect(broker_client).await;
        broker.subscribe("AAPL").await;
        loop {
            tokio::select! {
                data_opt = broker.next_market_update() => {
                    if let Some(data) = data_opt {
                        let _ = data_tx.send(data);
                    }
                    else {
                        event!(Level::WARN, "WebSocket connection closed. Terminating broker connection.");
                        break;
                    }
                }
                _ = broker_cancel.cancelled() => {
                    break;
                }
            }
        }
    });

    // Start order executor task
    let (order_tx, mut order_rx) = mpsc::unbounded_channel();
    let (order_update_tx, mut order_update_rx) = mpsc::unbounded_channel();
    let order_cancel = cancel.clone();
    let order_client = client.clone();
    tokio::spawn(async move {
        let mut order_executor = OrderExecutor::new(order_client).await;
        loop {
            tokio::select! {
                message = order_executor.next_order_update() => {
                    if let Some(data) = message {
                        let _ = order_update_tx.send(data.clone());
                        order_executor.on_order_update(data).await;
                    }
                    else {
                        event!(Level::WARN, "WebSocket connection closed. Terminating order executor connection.");
                        break;
                    }
                }
                order_opt = order_rx.recv() => {
                    if let Some(order) = order_opt {
                        order_executor.submit_order(order).await;
                    }
                }
                _ = order_cancel.cancelled() => {
                    break;
                }
            }
        }
    });

    // Start engine task
    let engine_cancel = cancel.clone();
    tokio::spawn(async move {
        let ctx = EngineContext::new(client);
        let engine = TradingEngine::new(ctx).await;
        loop {
            tokio::select! {
                data_opt = data_rx.recv() => {
                    event!(Level::INFO, "Market Update Received");
                    if let Some(data) = data_opt {
                        if let Some(order) = engine.on_market_data(data).await {
                            order_tx.send(order);
                        }
                    }
                }
                _ = order_update_rx.recv() => {
                    event!(Level::INFO, "Order Update Received");
                }
                _ = engine_cancel.cancelled() => {
                    break;
                }
            }
        }
    });

    if let Err(e) = tokio::signal::ctrl_c().await {
        event!(Level::ERROR, "Failed to wait for Ctrl-C. {}", e);
    }

    event!(Level::INFO, "Exiting Plutonic");
    cancel.cancel();

    Ok(())
}

fn init_environment() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .with_ansi(true)
        .init();

    event!(Level::INFO, "Initialized logging and environment");
}
