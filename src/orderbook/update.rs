use crate::{instrument::Instrument, pair::Pair};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub struct OrderbookUpdate {
    pub source: String,
    pub instrument: Instrument,
    pub pair: Pair,
    pub last_update_id: u64,
    pub bids: Vec<(f64, f64)>,
    pub asks: Vec<(f64, f64)>,
}
