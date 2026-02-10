use apca::data::v2::stream::{
    drive, Bar, Data, MarketData, Quote, RealtimeData, SymbolList, Trade, IEX,
};
use apca::{Client, Subscribable};
use futures::StreamExt;
use log::{error, info, warn};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_util::sync::{CancellationToken, DropGuard};

type DataStream = <RealtimeData<IEX> as Subscribable>::Stream;
type DataSubscription = <RealtimeData<IEX> as Subscribable>::Subscription;

#[derive(Debug, Clone, Copy)]
pub enum LiveData<B = Bar, Q = Quote, T = Trade> {
    Bar(B),
    Quote(Q),
    Trade(T),
}

impl From<Data> for LiveData {
    fn from(value: Data) -> Self {
        match value {
            Data::Bar(bar) => LiveData::Bar(bar),
            Data::Quote(quote) => LiveData::Quote(quote),
            Data::Trade(trade) => LiveData::Trade(trade),
            _ => unreachable!(),
        }
    }
}

/// Broker for listening to live data feeds.
///
/// The broker is the main entity responsible for interacting with an Exchange to
/// execute orders and get pricing for securities. This struct should only be used
/// as an I/O structure. It simply connects to the exchange and pushes data into our system.
pub struct AlpacaBroker {
    client: Arc<Client>,
    sender: Mutex<Option<Sender>>,
    data_tx: mpsc::UnboundedSender<LiveData>,
}

impl AlpacaBroker {
    #[allow(unused)]
    /// Constructs a new instance of an Alpaca Exchange.
    ///
    /// A live feed of data is <strong>not</strong> opened. See `connect` for
    /// how to open a connection.
    pub fn new(client: Arc<Client>, data_tx: mpsc::UnboundedSender<LiveData>) -> Self {
        Self {
            client,
            sender: Mutex::new(None),
            data_tx,
        }
    }

    /// Connect to the exchange and start listening to live data feeds.
    pub async fn connect(&self) {
        // Subscribe to realtime data, getting a stream (output) and subscription (control input)
        info!("Opening a connection with Alpaca");
        let (stream, subscription) = self.client.subscribe::<RealtimeData<IEX>>().await.unwrap();

        // TODO: Reader and Sender might not be the best names but OK for now
        // Create a Reader object to control the stream and a Sender responsible for sending messages to the server
        let (tx, rx) = mpsc::unbounded_channel();
        let cancel = CancellationToken::new();
        let connection = BrokerConnection {
            stream,
            subscription,
            command_rx: rx,
            data_tx: self.data_tx.clone(),
            cancel: cancel.clone(),
        };

        let sender = Sender {
            tx,
            _cancel: cancel.drop_guard(),
        };
        self.sender.lock().unwrap().replace(sender);

        // Spawn a task for the reader to continuosly read for data updates to publish into the bot
        tokio::spawn(Self::start(connection));
    }

    async fn start(mut connection: BrokerConnection) {
        // TODO: Pass a weak reference to the exchange which can be used to remove the reader on disconnect/call an on disconnect event
        loop {
            tokio::select! {
                Some(command) = connection.command_rx.recv() => {
                    match command {
                        BrokerCommand::Subscribe(symbols) => connection.subscribe(symbols).await,
                        BrokerCommand::Unsubscribe(symbols) => connection.unsubscribe(symbols).await,
                    };
                },
                websocket_response = connection.stream.next() => {
                    match websocket_response {
                        Some(websocket_data) => {
                            match websocket_data {
                                Ok(Ok(data)) => {
                                    let _ = connection.data_tx.send(data.into());
                                },
                                Ok(Err(err)) => {
                                    error!("Error parsing response {}", err);
                                },
                                Err(err) => {
                                    error!("Error reading websocket {}. Closing connection.", err);
                                    connection.cancel.cancel();
                                    break;
                                }
                            }
                        },
                        None => {
                            warn!("Connection with brokerage was closed.");
                            connection.cancel.cancel();
                            break;
                        }
                    }
                },
                _ = connection.cancel.cancelled() => break,
            }
        }
    }

    /// Disconnect from the exchange and close any open data feeds
    pub async fn disconnect(&self) {
        self.sender.lock().unwrap().take();
    }

    pub fn subscribe(&self, symbols: SymbolList) {
        let mut guard = self.sender.lock().unwrap();
        if let Some(ref sender) = *guard {
            if sender.tx.send(BrokerCommand::Subscribe(symbols)).is_err() {
                warn!("Websocket connection was closed.");
                guard.take();
            }
        }
    }

    pub fn unsubscribe(&self, symbols: SymbolList) {
        let mut guard = self.sender.lock().unwrap();
        if let Some(ref sender) = *guard {
            if sender.tx.send(BrokerCommand::Unsubscribe(symbols)).is_err() {
                warn!("Websocket connection was closed.");
                guard.take();
            }
        }
    }
}

enum BrokerCommand {
    Subscribe(SymbolList),
    Unsubscribe(SymbolList),
}

struct Sender {
    tx: mpsc::UnboundedSender<BrokerCommand>,
    _cancel: DropGuard,
}

struct BrokerConnection {
    stream: DataStream,
    subscription: DataSubscription,
    command_rx: mpsc::UnboundedReceiver<BrokerCommand>,
    data_tx: mpsc::UnboundedSender<LiveData>,
    cancel: CancellationToken,
}

impl BrokerConnection {
    async fn subscribe(&mut self, symbols: SymbolList) {
        let mut data = MarketData::default();
        data.set_bars(symbols.clone());
        data.set_quotes(symbols.clone());
        data.set_trades(symbols);

        let subscription_command = Box::pin(self.subscription.subscribe(&data));

        let () = drive(subscription_command, &mut self.stream)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    async fn unsubscribe(&mut self, symbols: SymbolList) {
        let mut data = MarketData::default();
        data.set_bars(symbols.clone());
        data.set_quotes(symbols.clone());
        data.set_trades(symbols);

        let unsubscribe_command = Box::pin(self.subscription.unsubscribe(&data));

        let () = drive(unsubscribe_command, &mut self.stream)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
}
