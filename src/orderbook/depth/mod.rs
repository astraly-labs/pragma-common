use crate::{web3::Chain, Instrument, Pair};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Depth {
    pub depth: DepthLevel,
    pub pair: Pair,
    pub source: String,
    pub chain: Option<Chain>,
    pub instrument: Instrument,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepthLevel {
    pub percentage: f64,
    pub bid: f64,
    pub ask: f64,
}
