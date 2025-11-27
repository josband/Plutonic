use std::{error::Error, sync::Arc};

use apca::{ApiInfo, Client};
use log::{debug, error, info, warn};
use plutonic::{engine::Plutonic, exchange::AlpacaExchange};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_environment();

    info!("Connecting to Alpaca API");
    let api_info = ApiInfo::from_env().inspect_err(|_| {
        error!("Environment is not properly set. Make sure to set both the API key and secret");
    })?;

    let client = Arc::new(Client::new(api_info));
    debug!(
        "Api key: {} Secret: {}",
        client.api_info().key_id,
        client.api_info().secret
    );

    let exchange = AlpacaExchange::new(client);

    let engine = Plutonic::new(exchange);

    engine.start().await;

    let _ = signal::ctrl_c().await.inspect_err(|err| {
        warn!("Failed to wait for SIGINT. Terminating session. {}", err);
    });

    info!("Shutting down Plutonic");

    Ok(())
}

fn init_environment() {
    simple_logger::init_with_level(log::Level::Debug).ok();
    dotenv::dotenv().ok();
    info!("Initializing logging and environment");
}
