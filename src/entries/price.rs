#[cfg(feature = "proto")]
use prost::Message;

use crate::{instrument_type::InstrumentType, pair::Pair, web3::Chain};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PriceEntry {
    pub source: String,
    pub chain: Option<Chain>,
    pub pair: Pair,
    pub timestamp_ms: i64,
    pub price: u128,
    pub volume: u128,
    pub expiration_timestamp: Option<i64>,
}

impl PriceEntry {
    pub fn instrument_type(&self) -> InstrumentType {
        match self.expiration_timestamp {
            None => InstrumentType::Spot,
            Some(_) => InstrumentType::Perp,
        }
    }
}

#[cfg(feature = "proto")]
impl PriceEntry {
    fn to_proto(&self) -> crate::schema::PriceEntry {
        crate::schema::PriceEntry {
            source: self.source.clone(),
            chain_option: match &self.chain {
                Some(chain) => Some(crate::schema::price_entry::ChainOption::Chain(
                    match chain {
                        Chain::Starknet => crate::schema::Chain::Starknet as i32,
                        Chain::Solana => crate::schema::Chain::Solana as i32,
                        Chain::Sui => crate::schema::Chain::Sui as i32,
                        Chain::Aptos => crate::schema::Chain::Aptos as i32,
                        Chain::Ethereum => crate::schema::Chain::Ethereum as i32,
                        Chain::Base => crate::schema::Chain::Base as i32,
                        Chain::Arbitrum => crate::schema::Chain::Arbitrum as i32,
                        Chain::Optimism => crate::schema::Chain::Optimism as i32,
                        Chain::ZkSync => crate::schema::Chain::Zksync as i32,
                        Chain::Polygon => crate::schema::Chain::Polygon as i32,
                        Chain::Bnb => crate::schema::Chain::Bnb as i32,
                        Chain::Avalanche => crate::schema::Chain::Avalanche as i32,
                        Chain::Gnosis => crate::schema::Chain::Gnosis as i32,
                        Chain::Worldchain => crate::schema::Chain::Worldchain as i32,
                    },
                )),
                None => Some(crate::schema::price_entry::ChainOption::NoChain(true)),
            },
            pair: Some(crate::schema::Pair {
                base: self.pair.base.clone(),
                quote: self.pair.quote.clone(),
            }),
            timestamp_ms: self.timestamp_ms,
            price: Some(crate::schema::UInt128 {
                low: self.price as u64,
                high: (self.price >> 64) as u64,
            }),
            volume: Some(crate::schema::UInt128 {
                low: self.volume as u64,
                high: (self.volume >> 64) as u64,
            }),
            expiration_option: Some(match self.expiration_timestamp {
                Some(ts) => crate::schema::price_entry::ExpirationOption::ExpirationTimestamp(ts),
                None => crate::schema::price_entry::ExpirationOption::NoExpiration(true),
            }),
        }
    }

    fn from_proto(proto: crate::schema::PriceEntry) -> Result<Self, prost::DecodeError> {
        let chain = match proto.chain_option {
            Some(crate::schema::price_entry::ChainOption::NoChain(_)) => None,
            Some(crate::schema::price_entry::ChainOption::Chain(chain)) => Some(match chain {
                x if x == crate::schema::Chain::Starknet as i32 => Chain::Starknet,
                x if x == crate::schema::Chain::Solana as i32 => Chain::Solana,
                x if x == crate::schema::Chain::Sui as i32 => Chain::Sui,
                x if x == crate::schema::Chain::Aptos as i32 => Chain::Aptos,
                x if x == crate::schema::Chain::Ethereum as i32 => Chain::Ethereum,
                x if x == crate::schema::Chain::Base as i32 => Chain::Base,
                x if x == crate::schema::Chain::Arbitrum as i32 => Chain::Arbitrum,
                x if x == crate::schema::Chain::Optimism as i32 => Chain::Optimism,
                x if x == crate::schema::Chain::Zksync as i32 => Chain::ZkSync,
                x if x == crate::schema::Chain::Polygon as i32 => Chain::Polygon,
                x if x == crate::schema::Chain::Bnb as i32 => Chain::Bnb,
                x if x == crate::schema::Chain::Avalanche as i32 => Chain::Avalanche,
                x if x == crate::schema::Chain::Gnosis as i32 => Chain::Gnosis,
                x if x == crate::schema::Chain::Worldchain as i32 => Chain::Worldchain,
                _ => {
                    return Err(prost::DecodeError::new(format!(
                        "Unknown chain value: {}",
                        chain
                    )))
                }
            }),
            None => {
                return Err(prost::DecodeError::new(
                    "Missing chain_option field in PriceEntry".to_string(),
                ))
            }
        };

        let pair = proto
            .pair
            .ok_or_else(|| prost::DecodeError::new("Missing pair field in PriceEntry"))?;
        let pair = Pair {
            base: pair.base,
            quote: pair.quote,
        };

        let price = proto
            .price
            .ok_or_else(|| prost::DecodeError::new("Missing price field in PriceEntry"))?;
        let price = (price.high as u128) << 64 | price.low as u128;

        let volume = proto
            .volume
            .ok_or_else(|| prost::DecodeError::new("Missing volume field in PriceEntry"))?;
        let volume = (volume.high as u128) << 64 | volume.low as u128;

        let expiration_timestamp = match proto.expiration_option {
            Some(crate::schema::price_entry::ExpirationOption::NoExpiration(_)) => None,
            Some(crate::schema::price_entry::ExpirationOption::ExpirationTimestamp(ts)) => Some(ts),
            None => {
                return Err(prost::DecodeError::new(
                    "Missing expiration_option field in PriceEntry".to_string(),
                ))
            }
        };

        Ok(PriceEntry {
            source: proto.source,
            chain,
            pair,
            timestamp_ms: proto.timestamp_ms,
            price,
            volume,
            expiration_timestamp,
        })
    }
}

#[cfg(feature = "proto")]
impl crate::ProtoSerialize for PriceEntry {
    fn to_proto_bytes(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto.encode_raw(&mut buf);
        buf
    }
}

#[cfg(feature = "proto")]
impl crate::ProtoDeserialize for PriceEntry {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        let proto = crate::schema::PriceEntry::decode(bytes)?;
        Self::from_proto(proto)
    }
}
