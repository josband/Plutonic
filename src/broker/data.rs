use apca::data::v2::stream::{Bar, Data, Quote, Trade};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BrokerData {
    Bar(Bar),
    Quote(Quote),
    Trade(Trade),
}

impl BrokerData {
    pub fn symbol(&self) -> &str {
        match self {
            BrokerData::Bar(bar) => &bar.symbol,
            BrokerData::Quote(quote) => &quote.symbol,
            BrokerData::Trade(trade) => &trade.symbol,
        }
    }
}

impl From<Data> for BrokerData {
    fn from(value: Data) -> Self {
        match value {
            Data::Bar(bar) => BrokerData::Bar(bar),
            Data::Quote(quote) => BrokerData::Quote(quote),
            Data::Trade(trade) => BrokerData::Trade(trade),
            _ => unreachable!(),
        }
    }
}
