use crate::{
    entries::{base::BaseEntry, EntryTrait},
    instrument_type::InstrumentType,
    pair::Pair,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub struct PerpEntry {
    pub base: BaseEntry,
    pub pair: Pair,
    pub price: u128,
    pub volume: u128,
}

impl EntryTrait for PerpEntry {
    fn base(&self) -> &BaseEntry {
        &self.base
    }

    fn pair(&self) -> &Pair {
        &self.pair
    }

    fn price(&self) -> u128 {
        self.price
    }

    fn volume(&self) -> u128 {
        self.volume
    }

    fn expiration_timestamp_ms(&self) -> Option<u64> {
        Some(0)
    }

    fn instrument_type(&self) -> InstrumentType {
        InstrumentType::Perp
    }
}

impl std::fmt::Display for PerpEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PERP[{}] {} @ {} (vol: {}) from {}/{}",
            self.pair,
            self.price,
            self.base.timestamp,
            self.volume,
            self.base.source,
            self.base.publisher
        )
    }
}
