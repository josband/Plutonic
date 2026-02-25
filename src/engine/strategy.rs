use apca::data::v2::stream::Bar;

use crate::engine::strategy::signal::Signal;

pub mod signal;

/// Trait encapsulating the core logic of a trading strategy.
pub trait Strategy {
    /// Process a market update.
    ///
    /// This method should be implemented by concrete strategy implementations to process incoming market data and generate trading signals.
    fn process(&self, data: &Bar) -> Signal;
}

/// A metric relating to an asset.
///
/// Indicators are useful for analyzing market trends and making informed trading decisions. Indicators
/// are used to generate trading signals and are an integral part of trading strategies.
pub trait Indicator {
    type Input;
    type Output;

    /// Updates the current value of the indicator
    fn update(&mut self, input: Self::Input);

    /// Retrieves the current value of the indicator
    fn value(&self) -> Self::Output;
}

/// Layer Responsible for the processing of live data.
///
/// The data processor should ingest live data as it comes in, calculate indicators needed for a strategy
pub struct StrategyExecutor<S: Strategy> {
    strategy: S,
}

impl<S: Strategy> StrategyExecutor<S> {
    pub fn new(strategy: S) -> Self {
        StrategyExecutor { strategy }
    }

    pub async fn evaluate_strategies(&self, data: &Bar) -> Signal {
        // Will become more complex as the executor evolves to handle multiple strategies and indicators
        self.strategy.process(data)
    }
}
