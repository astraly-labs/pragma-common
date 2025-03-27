pub mod base;

pub mod future;
pub mod perp;
pub mod spot;

use serde::{Serialize, Deserialize};

pub trait EntryTrait {
    fn base(&self) -> &base::BaseEntry;
    fn pair_id(&self) -> &String;
    fn price(&self) -> u128;
    fn volume(&self) -> u128;
    fn expiration_timestamp_ms(&self) -> Option<u64>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum MarketEntry {
    #[serde(rename = "spot")]
    Spot(spot::SpotEntry),
    #[serde(rename = "perp")]
    Perp(perp::PerpEntry),
    #[serde(rename = "future")]
    Future(future::FutureEntry),
}

impl EntryTrait for MarketEntry {
    fn base(&self) -> &base::BaseEntry {
        match self {
            Self::Spot(entry) => entry.base(),
            Self::Perp(entry) => entry.base(),
            Self::Future(entry) => entry.base(),
        }
    }

    fn pair_id(&self) -> &String {
        match self {
            Self::Spot(entry) => entry.pair_id(),
            Self::Perp(entry) => entry.pair_id(),
            Self::Future(entry) => entry.pair_id(),
        }
    }

    fn price(&self) -> u128 {
        match self {
            Self::Spot(entry) => entry.price(),
            Self::Perp(entry) => entry.price(),
            Self::Future(entry) => entry.price(),
        }
    }

    fn volume(&self) -> u128 {
        match self {
            Self::Spot(entry) => entry.volume(),
            Self::Perp(entry) => entry.volume(),
            Self::Future(entry) => entry.volume(),
        }
    }

    fn expiration_timestamp_ms(&self) -> Option<u64> {
        match self {
            Self::Spot(entry) => entry.expiration_timestamp_ms(),
            Self::Perp(entry) => entry.expiration_timestamp_ms(),
            Self::Future(entry) => entry.expiration_timestamp_ms(),
        }
    }
}

impl std::fmt::Display for MarketEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spot(entry) => write!(f, "spot: {entry}"),
            Self::Perp(entry) => write!(f, "perp: {entry}"),
            Self::Future(entry) => write!(f, "perp: {entry}"),
        }
    }
}
