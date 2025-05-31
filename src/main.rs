use std::{env, error::Error, time::Duration};

use apca::{api::v2::account, data::v2::stream::MarketData, ApiInfo, Client};
use log::{debug, error, info};
use plutonic::{AlpacaIngestor, DataIngestor, Service as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).ok();
    dotenv::dotenv().ok();
    info!("Initializing logging and environment");

    info!("Connecting to Alpaca API");
    let api_info = ApiInfo::from_env().inspect_err(|_| {
        error!("Environment is not properly set. Make sure to set both the API key and secret")
    })?;

    let client = Client::new(api_info);
    debug!(
        "Api key: {} Secret: {}",
        env::var("APCA_API_KEY_ID").unwrap(),
        env::var("APCA_API_SECRET_KEY").unwrap()
    );

    let account = client.issue::<account::Get>(&()).await.unwrap();
    info!("Account balance of ${}", account.cash);

    let ingestor = AlpacaIngestor::init(client).await;
    ingestor.start();

    tokio::time::sleep(Duration::from_secs(3)).await;
    info!("No Longer Sleeping");
    let mut data = MarketData::default();
    data.set_bars(["SPY"]);
    ingestor.subscribe(data);

    // TODO: Handle cancellation better
    tokio::signal::ctrl_c().await?;

    Ok(())
}
