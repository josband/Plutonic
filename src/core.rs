use std::sync::Arc;

use apca::{
    data::v2::stream::{drive, Bar, Data, MarketData, Quote, RealtimeData, Trade, IEX},
    Client, Subscribable,
};
use futures::StreamExt as _;
use log::{info, warn};
use tokio::{
    pin, select,
    sync::{broadcast, mpsc},
};
use tokio_util::sync::CancellationToken;

pub struct Plutonic {
    ingestor: MarketDataManager,
    shutdown_token: CancellationToken,
}

impl Plutonic {
    pub fn new() -> Self {
        todo!()
    }
}

enum Command {
    Subscribe(MarketData),
    Unsubscribe(MarketData),
}

/// Manager for underlying market data feed.
///
/// Reading of market data is performed by an background tokio task. The
/// `MarketDataManager` is the component responsible for controlling and
/// listening to that task.
pub struct MarketDataManager {
    cancel: CancellationToken,
    command_tx: mpsc::Sender<Command>,
    market_quote_rx: broadcast::Receiver<Quote>,
    market_bar_rx: broadcast::Receiver<Bar>,
    market_trade_rx: broadcast::Receiver<Trade>,
}

struct MarketDataIngestor {
    stream: <RealtimeData<IEX> as Subscribable>::Stream,
    subscription: <RealtimeData<IEX> as Subscribable>::Subscription,
    command_rx: mpsc::Receiver<Command>,
    quote_tx: broadcast::Sender<Quote>,
    bar_tx: broadcast::Sender<Bar>,
    trade_tx: broadcast::Sender<Trade>,
    cancel: CancellationToken
}

impl MarketDataIngestor {
    async fn init(
        client: Arc<Client>,
        command_rx: mpsc::Receiver<Command>,
        quote_tx: broadcast::Sender<Quote>,
        bar_tx: broadcast::Sender<Bar>,
        trade_tx: broadcast::Sender<Trade>,
        cancel: CancellationToken
    ) -> Self {
        // Connect to live market data, default to no data but establish the connection
        let (stream, subscription) =
            client.subscribe::<RealtimeData<IEX>>().await.unwrap();
        info!("Connected to brokerage");
        
        Self {
            stream,
            subscription,
            command_rx,
            quote_tx,
            bar_tx,
            trade_tx,
            cancel
        }
    }

    async fn run(&mut self) {
        // TODO: Is data stream is cancellation safe? Appears to be cancel safe though based on apca impl
        loop {
            select! {
                Some(message_result) = self.stream.next() => {
                    match message_result {
                        Ok(Ok(data)) => {
                            match data {
                                Data::Bar(bar) => {
                                    if self.bar_tx.receiver_count() > 0 {
                                        let _ = self.bar_tx.send(bar);
                                    }
                                },
                                Data::Quote(quote) => {
                                    if self.quote_tx.receiver_count() > 0 {
                                        let _ = self.quote_tx.send(quote);
                                    }
                                },
                                Data::Trade(trade) => {
                                    if self.trade_tx.receiver_count() > 0 {
                                        let _ = self.trade_tx.send(trade);
                                    }
                                }
                                _ => ()
                            }
                        },
                        // TODO: Give more detailed warning log
                        Ok(Err(_)) => {
                            warn!("Failed to parse stream message");
                        },
                        Err(_) => {
                            warn!("Encountered error with websocket connection");
                        },
                    }
                },
                Some(command) = self.command_rx.recv() => {
                    match command {
                        Command::Subscribe(data) => {
                            // Issue control message to data service and await response
                            let subscribe_future = self.subscription.subscribe(&data);
                            pin!(subscribe_future);
                            let () = drive(subscribe_future, &mut self.stream)
                                .await
                                .unwrap()
                                .unwrap()
                                .unwrap();

                            info!("Successfully updated live market data feed");
                        },
                        Command::Unsubscribe(data) => {
                            // Issue control message to data service and await response
                            let subscribe_future = self.subscription.unsubscribe(&data);
                            pin!(subscribe_future);
                            let () = drive(subscribe_future, &mut self.stream)
                                .await
                                .unwrap()
                                .unwrap()
                                .unwrap();

                            info!("Successfully updated live market data feed");
                        }
                    }
                },
                _ = self.cancel.cancelled() => {
                    info!("Disconnecting from live data source");
                    break;
                },
            }
        }
    }
}

impl MarketDataManager {
    /// Connects to a live market data source.
    pub fn connect(client: Arc<Client>) -> Self {
        let cancel = CancellationToken::new();
        let (command_tx, mut command_rx) = mpsc::channel(16);
        let (market_quote_tx, market_quote_rx) = broadcast::channel(128);
        let (market_bar_tx, market_bar_rx) = broadcast::channel(1024);
        let (market_trade_tx, market_trade_rx) = broadcast::channel(1024);

        let cancel_token = cancel.clone();
        tokio::spawn(async move {
            // Connect to live market data, default to no data but establish the connection
            let (mut stream, mut subscription) =
                client.subscribe::<RealtimeData<IEX>>().await.unwrap();
            info!("Connected to brokerage");

            // TODO: Is data stream is cancellation safe? Appears to be cancel safe though based on apca impl
            loop {
                select! {
                    Some(message_result) = stream.next() => {
                        match message_result {
                            Ok(Ok(data)) => {
                                match data {
                                    Data::Bar(bar) => {
                                        if market_bar_tx.receiver_count() > 0 {
                                            let _ = market_bar_tx.send(bar);
                                        }
                                    },
                                    Data::Quote(quote) => {
                                        if market_quote_tx.receiver_count() > 0 {
                                            let _ = market_quote_tx.send(quote);
                                        }
                                    },
                                    Data::Trade(trade) => {
                                        if market_trade_tx.receiver_count() > 0 {
                                            let _ = market_trade_tx.send(trade);
                                        }
                                    }
                                    _ => ()
                                }
                            },
                            // TODO: Give more detailed warning log
                            Ok(Err(_)) => {
                                warn!("Failed to parse stream message");
                            },
                            Err(_) => {
                                warn!("Encountered error with websocket connection");
                            },
                        }
                    },
                    Some(command) = command_rx.recv() => {
                        match command {
                            Command::Subscribe(data) => {
                                // Issue control message to data service and await response
                                let subscribe_future = subscription.subscribe(&data);
                                pin!(subscribe_future);
                                let () = drive(subscribe_future, &mut stream)
                                    .await
                                    .unwrap()
                                    .unwrap()
                                    .unwrap();

                                info!("Successfully updated live market data feed");
                            },
                            Command::Unsubscribe(data) => {
                                // Issue control message to data service and await response
                                let subscribe_future = subscription.unsubscribe(&data);
                                pin!(subscribe_future);
                                let () = drive(subscribe_future, &mut stream)
                                    .await
                                    .unwrap()
                                    .unwrap()
                                    .unwrap();

                                info!("Successfully updated live market data feed");
                            }
                        }
                    },
                    _ = cancel_token.cancelled() => {
                        info!("Disconnecting from live data source");
                        break;
                    },
                }
            }
        });

        Self {
            cancel,
            command_tx,
            market_quote_rx,
            market_bar_rx,
            market_trade_rx,
        }
    }

    pub fn add_market_subscription(&self, data: MarketData) {
        let _ = self.command_tx.send(Command::Subscribe(data));
    }

    pub fn remove_market_subscription(&self, data: MarketData) {
        let _ = self.command_tx.send(Command::Unsubscribe(data));
    }

    /// Subscribe to a market feed of live quotes.
    ///
    /// Subscribes to a live feed of market quotes starting from the time of subscription. Multiple
    /// subscribers can exist at any point in time. Publishes by the `MarketDataIngestor` are
    /// one-to-many. Every subscriber will receive the data.
    pub fn subscribe_quote(&self) -> broadcast::Receiver<Quote> {
        self.market_quote_rx.resubscribe()
    }

    /// Subscribe to a market feed of live bars.
    ///
    /// Subscribes to a live feed of market bars starting from the time of subscription. Multiple
    /// subscribers can exist at any point in time. Publishes by the `MarketDataIngestor` are
    /// one-to-many. Every subscriber will receive the data.
    pub fn subscribe_bar(&self) -> broadcast::Receiver<Bar> {
        self.market_bar_rx.resubscribe()
    }

    /// Subscribe to a market feed of live trades.
    ///
    /// Subscribes to a live feed of market trades starting from the time of subscription. Multiple
    /// subscribers can exist at any point in time. Publishes by the `MarketDataIngestor` are
    /// one-to-many. Every subscriber will receive the data.
    pub fn subscribe_trade(&self) -> broadcast::Receiver<Trade> {
        self.market_trade_rx.resubscribe()
    }
}

impl Service for MarketDataManager {
    fn start(&self) {
        todo!();
    }

    fn stop(&self) {
        self.cancel.cancel();
    }
}

impl Drop for MarketDataManager {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

pub trait Service {
    fn start(&self);

    fn stop(&self);
}

// pub trait DataIngestor: Service {
//     fn subscribe(&self, data: MarketData);

//     fn unsubscribe(&self, data: MarketData);
// }
