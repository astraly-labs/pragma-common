use crate::entries::{base::BaseEntry, EntryTrait};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub struct FutureEntry {
    pub base: BaseEntry,
    pub pair_id: String,
    pub price: u128,
    pub volume: u128,
    pub expiration_timestamp_ms: u64,
}

impl EntryTrait for FutureEntry {
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
        Some(self.expiration_timestamp_ms)
    }
}

impl std::fmt::Display for FutureEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FUTURE[{}] {} @ {} (vol: {}, exp: {}) from {}/{}",
            self.pair_id,
            self.price,
            self.base.timestamp,
            self.volume,
            self.expiration_timestamp_ms,
            self.base.source,
            self.base.publisher
        )
    }
}
