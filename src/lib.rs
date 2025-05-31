use apca::{data::v2::stream::{drive, MarketData, RealtimeData, IEX}, Client };
use futures::StreamExt as _;
use log::{info, warn};
use tokio::{select, sync::mpsc::{self, UnboundedSender}};
use tokio_util::sync::CancellationToken;

pub trait Service {
    fn start(&self);

    fn stop(&self);

    fn shutdown(&self);
}

pub trait DataIngestor: Service {
    fn subscribe(&self, data: MarketData);

    fn unsubscribe(&self, data: MarketData);
}

#[allow(dead_code)]
pub struct AlpacaIngestor {
    client: Client,
    tx: UnboundedSender<Command>,
    cancel_token: CancellationToken
}

enum Command {
    Subscribe(MarketData),
    Unsubscribe(MarketData),
    Start,
    Stop
}

impl AlpacaIngestor {
    pub async fn init(client: Client) -> Self {
        let cancel_token = CancellationToken::new();
        let (mut stream, mut subscription) = client
            .subscribe::<RealtimeData<IEX>>()
            .await
            .unwrap();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let service_cancel_token = cancel_token.clone();
        let _service_future = tokio::spawn(async move {
            // Wait for initial run command
            let mut running;
            loop {
                if let Some(Command::Start) = rx.recv().await {
                    info!("Starting up Alpaca Data Ingestion.");
                    running = true;
                    break;
                } else {
                    warn!("Data Ingestor has not been started. Start the ingestion layer to process other commands.");
                }
            }

            loop {
                if !running { 
                    continue
                }

                select! {
                    _ = service_cancel_token.cancelled() => break,
                    Some(_data_result) = stream.next() => {
                        info!("Received Live Market Data.");
                    },
                    Some(command) = rx.recv() => {
                        match command {
                            Command::Subscribe(data) => {
                                info!("Subscribing to new Market Data.");
                                let subscribe_future = Box::pin(subscription.subscribe(&data));
                                let () = drive(subscribe_future, &mut stream)
                                    .await
                                    .unwrap()
                                    .unwrap()
                                    .unwrap();
                            },
                            Command::Unsubscribe(data) => {
                                info!("Unsubscribing from Market Data.");
                                let subscribe_future = Box::pin(subscription.unsubscribe(&data));
                                let () = drive(subscribe_future, &mut stream)
                                    .await
                                    .unwrap()
                                    .unwrap()
                                    .unwrap();
                            },
                            Command::Start => running = true,
                            Command::Stop => running = false
                        }
                    }
                }
            }
        });

        Self {
            client,
            tx,
            cancel_token
        }
    }
}

impl Drop for AlpacaIngestor {
    fn drop(&mut self) {
        if !self.cancel_token.is_cancelled() {
            self.cancel_token.cancel();
        }
    }
}

impl Service for AlpacaIngestor {
    fn start(&self) {
        let _ = self.tx.send(Command::Start);
    }

    fn stop(&self) {
        let _ = self.tx.send(Command::Stop);
    }

    fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}

impl DataIngestor for AlpacaIngestor {
    fn subscribe(&self, data: MarketData) {
        let _ = self.tx.send(Command::Subscribe(data));
    }

    fn unsubscribe(&self, data: MarketData) {
        let _ = self.tx.send(Command::Unsubscribe(data));
    }
}