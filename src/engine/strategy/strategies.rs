use apca::data::v2::stream::Bar;
use async_trait::async_trait;
use num_decimal::Num;

use crate::engine::{
    signal::{Signal, SignalDirection},
    Indicator, Strategy,
};

/// Trend side indicating whether a set of indicators are trending up or down.
#[derive(Debug, Clone, Copy)]
enum Trend {
    Long,
    Short,
}

/// Simple SMA Crossover Strategy
///
/// Provide two periods for two SMA indicators. When the periods cross over
/// one another, a signal is generated. Otherwise, the strategy holds.
pub struct SmaCrossoverStrategy {
    fast_ma: SimpleMovingAverage,
    slow_ma: SimpleMovingAverage,
    last_cross_trend: Option<Trend>,
}

impl SmaCrossoverStrategy {
    pub fn new(fast_period: u32, slow_period: u32) -> Self {
        SmaCrossoverStrategy {
            fast_ma: SimpleMovingAverage::new(fast_period),
            slow_ma: SimpleMovingAverage::new(slow_period),
            last_cross_trend: None,
        }
    }
}

#[async_trait]
impl Strategy for SmaCrossoverStrategy {
    async fn process(&mut self, bar: &Bar) -> Signal {
        self.fast_ma.update(bar.clone());
        self.slow_ma.update(bar.clone());

        match (self.fast_ma.value(), self.slow_ma.value()) {
            (Some(fast), Some(slow)) => {
                if fast > slow {
                    if let Some(Trend::Short) = self.last_cross_trend {
                        return Signal::new(bar.symbol.clone(), SignalDirection::Buy);
                    }

                    self.last_cross_trend = Some(Trend::Long);
                } else {
                    if let Some(Trend::Long) = self.last_cross_trend {
                        return Signal::new(bar.symbol.clone(), SignalDirection::Sell);
                    }

                    self.last_cross_trend = Some(Trend::Short);
                };

                Signal::new(bar.symbol.clone(), SignalDirection::Neutral)
            }
            _ => Signal::new(bar.symbol.clone(), SignalDirection::Neutral),
        }
    }
}

struct SimpleMovingAverage {
    period: u32,
    prices: Vec<Num>,
    value: Option<Num>,
}

impl SimpleMovingAverage {
    pub fn new(period: u32) -> Self {
        SimpleMovingAverage {
            period,
            prices: Vec::new(),
            value: None,
        }
    }
}

impl Indicator for SimpleMovingAverage {
    type Input = Bar;
    type Output = Option<Num>;

    fn update(&mut self, input: Self::Input) {
        self.prices.push(input.close_price.clone());
        if self.prices.len() > self.period as usize {
            self.prices.remove(0);
        }

        if self.prices.len() == self.period as usize {
            self.value = Some(
                self.prices.iter().fold(Num::from(0), |acc, n| acc + n)
                    / Num::from(self.prices.len()),
            );
        }
    }

    fn value(&self) -> Self::Output {
        self.value.clone()
    }
}
