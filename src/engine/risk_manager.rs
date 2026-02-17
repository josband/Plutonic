use crate::engine::Signal;

pub struct RiskManager {}

impl RiskManager {
    pub fn check_order_safety(&self) -> Option<Signal> {
        None
    }
}
