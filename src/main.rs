use std::sync::Arc;

use apca::{ApiInfo, Client};
use log::{debug, error, info};
use plutonic::{
    broker::AlpacaBroker,
    engine::{EngineContext, TradingEngine},
    order_executor::OrderExecutor,
};
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_environment();

    info!("Building Alpaca API Information");
    let api_info = ApiInfo::from_env().inspect_err(|_| {
        error!("Environment is not properly set. Make sure to set both the API key and secret");
    })?;

    let client = Arc::new(Client::new(api_info));
    debug!(
        "Api key: {} Secret: {}",
        client.api_info().key_id,
        client.api_info().secret
    );

    info!("Starting Plutonic");

    // **************************** REMOVE ****************************
    let mut order_executor = OrderExecutor::new(client.clone()).await;
    let mut broker = AlpacaBroker::connect(client.clone()).await;
    let _ = TradingEngine::new(EngineContext::new(client));
    broker.subscribe("AAPL").await;
    loop {
        tokio::select! {
            Some(data) = broker.next() => {
                info!("Received market data {:?}", data);
            },
            Some(data) = order_executor.next_order_update() => {
                info!("Received order update {:?}", data);
            },
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C signal");
                break;
            }
        }
    }
    // ****************************************************************

    info!("Exiting Plutonic");

    Ok(())
}

fn init_environment() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    info!("Initialized logging and environment");
}
