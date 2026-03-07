use std::collections::HashMap;

use apca::data::v2::stream::Bar;
use async_trait::async_trait;
use futures::StreamExt;

use crate::engine::{signal::SignalDirection, strategy::signal::Signal};

pub mod signal;
pub mod strategies;

/// Trait encapsulating the core logic of a trading strategy.
#[async_trait]
pub trait Strategy: Send + Sync + 'static {
    /// Process a market update.
    ///
    /// This method should be implemented by concrete strategy implementations to process incoming market data and generate trading signals.
    async fn process(&mut self, bar: &Bar) -> Signal;
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

    pub async fn evaluate_strategies(&mut self, bar: &Bar) -> Signal {
        // TODO: Make this concurrent by using `map` and `buffered` instead of `then`
        let counts = futures::stream::iter(&mut self.strategies)
            .then(|s| async move {
                let bar = bar.clone();
                s.process(&bar).await
            })
            .fold(HashMap::new(), |mut counts, s| async move {
                *counts.entry(s.direction()).or_insert(0) += 1;
                counts
            })
            .await;

        let max_count = counts.values().max().copied().unwrap_or(0);
        let winners: Vec<_> = counts.iter().filter(|(_, &v)| v == max_count).collect();
        let dir = if winners.len() == 1 {
            *winners[0].0
        } else {
            SignalDirection::Neutral
        };

        Signal::new(bar.symbol.clone(), dir)
    }
}

impl Default for StrategyExecutor {
    fn default() -> Self {
        Self::new()
    }
}
