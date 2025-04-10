pub mod base;
pub mod perp;
pub mod spot;

pub use base::*;
pub use perp::*;
pub use spot::*;

use crate::{instrument_type::InstrumentType, pair::Pair};

pub trait EntryTrait {
    fn base(&self) -> &base::BaseEntry;
    fn pair(&self) -> &Pair;
    fn price(&self) -> u128;
    fn volume(&self) -> u128;
    fn expiration_timestamp_ms(&self) -> Option<u64>;
    fn instrument_type(&self) -> InstrumentType;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
pub enum MarketEntry {
    #[cfg_attr(feature = "serde", serde(rename = "spot"))]
    Spot(spot::SpotEntry),
    #[cfg_attr(feature = "serde", serde(rename = "perp"))]
    Perp(perp::PerpEntry),
}

impl EntryTrait for MarketEntry {
    fn instrument_type(&self) -> InstrumentType {
        match self {
            Self::Spot(entry) => entry.instrument_type(),
            Self::Perp(entry) => entry.instrument_type(),
        }
    }

    fn base(&self) -> &base::BaseEntry {
        match self {
            Self::Spot(entry) => entry.base(),
            Self::Perp(entry) => entry.base(),
        }
    }

    fn pair(&self) -> &Pair {
        match self {
            Self::Spot(entry) => entry.pair(),
            Self::Perp(entry) => entry.pair(),
        }
    }

    fn price(&self) -> u128 {
        match self {
            Self::Spot(entry) => entry.price(),
            Self::Perp(entry) => entry.price(),
        }
    }

    fn volume(&self) -> u128 {
        match self {
            Self::Spot(entry) => entry.volume(),
            Self::Perp(entry) => entry.volume(),
        }
    }

    fn expiration_timestamp_ms(&self) -> Option<u64> {
        match self {
            Self::Spot(entry) => entry.expiration_timestamp_ms(),
            Self::Perp(entry) => entry.expiration_timestamp_ms(),
        }
    }
}

impl std::fmt::Display for MarketEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spot(entry) => write!(f, "spot: {entry}"),
            Self::Perp(entry) => write!(f, "perp: {entry}"),
        }
    }
}

impl From<MarketEntry> for InstrumentType {
    fn from(value: MarketEntry) -> Self {
        value.instrument_type()
    }
}
