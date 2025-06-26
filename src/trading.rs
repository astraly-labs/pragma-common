#[cfg(feature = "serde")]
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer};

#[derive(
    Clone, Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Copy, strum::EnumString, strum::Display,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
pub enum Side {
    Long,
    Short,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Long => Side::Short,
            Side::Short => Side::Long,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Side::from_str(&s).map_err(serde::de::Error::custom)
    }
}
