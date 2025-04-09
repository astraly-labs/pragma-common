use std::{fmt::Display, str::FromStr};

use crate::entries::MarketEntry;

#[derive(Debug, thiserror::Error)]
pub enum InstrumentError {
    #[error("Invalid instrument id")]
    InvalidId,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub enum Instrument {
    Spot,
    Perp,
}

impl Instrument {
    pub const ALL: [Self; 2] = [Self::Spot, Self::Perp];

    pub const fn is_spot(&self) -> bool {
        match self {
            Self::Spot => true,
            Self::Perp => false,
        }
    }

    pub const fn is_perp(&self) -> bool {
        match self {
            Self::Spot => false,
            Self::Perp => true,
        }
    }
}

impl Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spot => write!(f, "spot"),
            Self::Perp => write!(f, "perp"),
        }
    }
}

impl From<&MarketEntry> for Instrument {
    fn from(value: &MarketEntry) -> Self {
        match value {
            MarketEntry::Spot(_) => Self::Spot,
            MarketEntry::Perp(_) => Self::Perp,
        }
    }
}

impl TryFrom<i32> for Instrument {
    type Error = InstrumentError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Spot),
            2 => Ok(Self::Perp),
            _ => Err(InstrumentError::InvalidId),
        }
    }
}

impl FromStr for Instrument {
    type Err = InstrumentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spot" => Ok(Self::Spot),
            "perp" => Ok(Self::Perp),
            _ => Err(InstrumentError::InvalidId),
        }
    }
}
