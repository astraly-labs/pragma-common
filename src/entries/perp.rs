use serde::{Serialize, Deserialize};

use crate::entries::{EntryTrait, base::BaseEntry};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerpEntry {
    pub base: BaseEntry,
    pub pair_id: String,
    pub price: u128,
    pub volume: u128,
}

impl EntryTrait for PerpEntry {
    fn base(&self) -> &BaseEntry {
        &self.base
    }

    fn pair_id(&self) -> &String {
        &self.pair_id
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
}

impl std::fmt::Display for PerpEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PERP[{}] {} @ {} (vol: {}) from {}/{}",
            self.pair_id,
            self.price,
            self.base.timestamp,
            self.volume,
            self.base.source,
            self.base.publisher
        )
    }
}
