use std::collections::HashMap;

use apca::data::v2::stream::Bar;

use crate::engine::{signal::SignalDirection, strategy::signal::Signal};

pub mod signal;

/// Trait encapsulating the core logic of a trading strategy.
pub trait Strategy: Send + Sync + 'static {
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
pub struct StrategyExecutor {
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyExecutor {
    pub fn new() -> Self {
        StrategyExecutor { strategies: vec![] }
    }

    pub fn register_strategy<S: Strategy>(&mut self, strategy: S) {
        self.strategies.push(Box::new(strategy));
    }

    pub async fn evaluate_strategies(&self, data: &Bar) -> Signal {
        let mut counts = HashMap::new();
        self.strategies.iter().for_each(|s| {
            *counts.entry(s.process(data).direction()).or_insert(0) += 1;
        });

        let dir = counts
            .iter()
            .max_by_key(|&(_, v)| *v)
            .map(|(k, _)| *k)
            .unwrap_or(SignalDirection::Neutral);

        Signal::new(data.symbol.clone(), dir)
    }
}

impl Default for StrategyExecutor {
    fn default() -> Self {
        Self::new()
    }
}
