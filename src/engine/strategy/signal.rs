use apca::api::v2::order::Side;

/// Trading Signal.
///
/// Trading signals are automated, data-driven triggers that indicate an
/// action to be taken by a trading engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    symbol: String,
    direction: SignalDirection,
}

impl Signal {
    /// Create a new trading signal.
    pub fn new(symbol: String, direction: SignalDirection) -> Self {
        Signal { symbol, direction }
    }

    /// Get the direction of the signal.
    pub fn direction(&self) -> SignalDirection {
        self.direction
    }
}

/// Trading Signal Direction
///
/// Directions indicate the particular action to be taken. This
/// can be either to buy, sell, or hold a particular asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalDirection {
    Buy,
    Neutral,
    Sell,
}

impl SignalDirection {
    pub fn order_side(&self) -> Option<Side> {
        match self {
            SignalDirection::Buy => Some(Side::Buy),
            SignalDirection::Sell => Some(Side::Sell),
            SignalDirection::Neutral => None,
        }
    }
}
