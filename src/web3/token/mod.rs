pub mod constants;

pub use constants::*;

use std::collections::BTreeMap;

use super::Chain;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Token {
    pub name: String,
    pub ticker: String,
    pub decimals: u32,
    pub addresses: Option<BTreeMap<Chain, String>>,
}

impl Token {
    pub fn new(
        name: &str,
        ticker: &str,
        decimals: u32,
        addresses: Option<BTreeMap<Chain, String>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            ticker: ticker.to_string(),
            decimals,
            addresses,
        }
    }

    pub fn new_without_addresses(name: &str, ticker: &str, decimals: u32) -> Self {
        Self {
            name: name.to_string(),
            ticker: ticker.to_string(),
            decimals,
            addresses: None,
        }
    }

    #[must_use]
    pub fn with_addresses(mut self, addresses: BTreeMap<Chain, String>) -> Self {
        self.addresses = Some(addresses);
        self
    }

    /// Returns the address of the token for the provided `Chain`
    pub fn address(&self, chain: Chain) -> Option<String> {
        self.addresses.as_ref().and_then(|e| e.get(&chain).cloned())
    }
}
