use apca::data::v2::stream::{drive, MarketData, RealtimeData, IEX};
use apca::{Client, Subscribable};
use futures::StreamExt;
use std::pin::pin;
use std::sync::Arc;
use tracing::{event, Level};

use crate::broker::data::BrokerData;

pub mod data;

// TODO: impl next() or something like that for AlpacaBroker

type LiveDataStream = <RealtimeData<IEX> as Subscribable>::Stream;
type LiveDataSubscription = <RealtimeData<IEX> as Subscribable>::Subscription;

pub struct AlpacaBroker {
    stream: LiveDataStream,
    subscription: LiveDataSubscription,
}

impl AlpacaBroker {
    pub async fn connect(client: Arc<Client>) -> Self {
        // Subscribe to realtime data, getting a stream (output) and subscription (control input)
        event!(Level::INFO, "Opening a connection with Alpaca");
        let (stream, subscription) = client.subscribe::<RealtimeData<IEX>>().await.unwrap();

        Self {
            stream,
            subscription,
        }
    }

    pub async fn subscribe(&mut self, symbol: &'static str) {
        let mut new_subs = MarketData::default();
        new_subs.set_bars([symbol]);
        new_subs.set_quotes([symbol]);
        new_subs.set_trades([symbol]);

        let subscription_command = Box::pin(self.subscription.subscribe(&new_subs));

        let () = drive(subscription_command, &mut self.stream)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    pub async fn unsubscribe(&mut self, symbol: &'static str) {
        let mut new_subs = MarketData::default();
        new_subs.set_bars([symbol]);
        new_subs.set_quotes([symbol]);
        new_subs.set_trades([symbol]);

        let subscription_command = pin!(self.subscription.unsubscribe(&new_subs));

        let () = drive(subscription_command, &mut self.stream)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    pub async fn next_market_update(&mut self) -> Option<BrokerData> {
        self.stream.next().await.map(|d| d.unwrap().unwrap().into())
    }
}
