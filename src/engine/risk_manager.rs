#![allow(unused)]

use apca::api::v2::order::CreateReq;

#[derive(Debug)]
pub enum RiskDecision {
    Accept,
    Reject(String),
    Modify(Box<CreateReq>),
}

pub struct RiskManager {}

impl RiskManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate_order(&self, order: &CreateReq) -> RiskDecision {
        RiskDecision::Reject(String::from("Not implemented"))
    }
}
