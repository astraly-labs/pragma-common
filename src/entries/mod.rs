pub mod base;

pub mod perp;
pub mod spot;

pub trait EntryTrait {
    fn base(&self) -> &base::BaseEntry;
    fn pair_id(&self) -> &String;
    fn price(&self) -> u128;
    fn volume(&self) -> u128;
    fn expiration_timestamp_ms(&self) -> Option<u64>;
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
    fn base(&self) -> &base::BaseEntry {
        match self {
            Self::Spot(entry) => entry.base(),
            Self::Perp(entry) => entry.base(),
        }
    }

    fn pair_id(&self) -> &String {
        match self {
            Self::Spot(entry) => entry.pair_id(),
            Self::Perp(entry) => entry.pair_id(),
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
