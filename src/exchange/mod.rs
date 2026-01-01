use apca::data::v2::stream::{
    drive, Bar, Data, MarketData, Quote, RealtimeData, SymbolList, Trade, IEX,
};
use apca::Subscribable;
use futures::StreamExt;
use log::{error, info, warn};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tokio::select;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::{CancellationToken, DropGuard};

use crate::engine::EngineContext;

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

/// Interface to interact with the Alpaca Exchange.
pub struct AlpacaExchange {
    engine_ctx: Arc<EngineContext>,
    sender: Mutex<Option<Sender>>,
    data_tx: broadcast::Sender<LiveData>,
}

impl AlpacaExchange {
    /// Constructs a new instance of an Alpaca Exchange.
    ///
    /// A live feed of data is <strong>not</strong> opened. See `connect` for
    /// how to open a connection.
    pub fn new(ctx: Arc<EngineContext>) -> Self {
        Self {
            data_tx: ctx.sender().clone(),
            engine_ctx: ctx,
            sender: Mutex::new(None),
        }
    }

    /// Connect to the exchange and start listening to live data feeds.
    pub async fn connect(&self) {
        // Subscribe to realtime data, getting a stream (output) and subscription (control input)
        info!("Opening a connection with Alpaca");
        let (stream, subscription) = self
            .engine_ctx
            .client
            .subscribe::<RealtimeData<IEX>>()
            .await
            .unwrap();

        // TODO: Reader and Sender might not be the best names but OK for now
        // Create a Reader object to control the stream and a Sender responsible for sending messages to the server
        let (tx, rx) = mpsc::unbounded_channel();
        let cancel = CancellationToken::new();
        let connection = Connection {
            ctx: self.engine_ctx.clone(),
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
        self.engine_ctx.is_running.store(true, Ordering::SeqCst);

        // Spawn a task for the reader to continuosly read for data updates to publish into the bot
        tokio::spawn(Self::start(connection));
    }

    // TODO: Pass a weak reference to the exchange which can be used to remove the reader on disconnect/call an on disconnect event
    async fn start(mut connection: Connection) {
        loop {
            select! {
                Some(command) = connection.command_rx.recv() => {
                    match command {
                        Command::Subscribe(symbols) => connection.subscribe(symbols).await,
                        Command::Unsubscribe(symbols) => connection.unsubscribe(symbols).await,
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

        // Connection has terminated if loop exited, update context
        connection.ctx.is_running.store(false, Ordering::SeqCst);
    }

    /// Disconnect from the exchange and close any open data feeds
    pub async fn disconnect(&self) {
        self.sender.lock().unwrap().take();
    }

    pub fn subscribe(&self, symbols: SymbolList) {
        let mut guard = self.sender.lock().unwrap();
        if let Some(ref sender) = *guard {
            if sender.tx.send(Command::Subscribe(symbols)).is_err() {
                warn!("Websocket connection was closed.");
                guard.take();
            }
        }
    }

    pub fn unsubscribe(&self, symbols: SymbolList) {
        let mut guard = self.sender.lock().unwrap();
        if let Some(ref sender) = *guard {
            if sender.tx.send(Command::Unsubscribe(symbols)).is_err() {
                warn!("Websocket connection was closed.");
                guard.take();
            }
        }
    }
}

enum Command {
    Subscribe(SymbolList),
    Unsubscribe(SymbolList),
}

struct Sender {
    tx: mpsc::UnboundedSender<Command>,
    _cancel: DropGuard,
}

struct Connection {
    ctx: Arc<EngineContext>,
    stream: DataStream,
    subscription: DataSubscription,
    command_rx: mpsc::UnboundedReceiver<Command>,
    data_tx: broadcast::Sender<LiveData>,
    cancel: CancellationToken,
}

impl Connection {
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
