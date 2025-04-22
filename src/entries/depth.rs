use crate::{instrument_type::InstrumentType, web3::Chain, Pair};
#[cfg(feature = "proto")]
use prost::Message;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DepthEntry {
    pub source: String,
    pub chain: Option<Chain>,
    pub instrument_type: InstrumentType,
    pub pair: Pair,
    pub timestamp_ms: i64,
    pub depth: DepthLevel,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DepthLevel {
    pub percentage: f64,
    pub bid: f64,
    pub ask: f64,
}

#[cfg(feature = "proto")]
impl DepthEntry {
    fn to_proto(&self) -> crate::schema::DepthEntry {
        crate::schema::DepthEntry {
            source: self.source.clone(),
            instrument_type: match self.instrument_type {
                InstrumentType::Spot => crate::schema::InstrumentType::Spot,
                InstrumentType::Perp => crate::schema::InstrumentType::Perp,
            } as i32,
            pair: Some(crate::schema::Pair {
                base: self.pair.base.clone(),
                quote: self.pair.quote.clone(),
            }),
            depth: Some(crate::schema::DepthLevel {
                percentage: self.depth.percentage,
                bid: self.depth.bid,
                ask: self.depth.ask,
            }),
            chain_option: Some(match &self.chain {
                Some(chain) => crate::schema::depth_entry::ChainOption::Chain(match chain {
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
                }),
                None => crate::schema::depth_entry::ChainOption::NoChain(true),
            }),
            timestamp_ms: self.timestamp_ms,
        }
    }

    fn from_proto(proto: crate::schema::DepthEntry) -> Result<Self, prost::DecodeError> {
        let instrument_type = match proto.instrument_type() {
            crate::schema::InstrumentType::Spot => InstrumentType::Spot,
            crate::schema::InstrumentType::Perp => InstrumentType::Perp,
        };

        let pair = proto
            .pair
            .ok_or_else(|| prost::DecodeError::new("Missing pair field in DepthEntry"))?;
        let pair = Pair {
            base: pair.base,
            quote: pair.quote,
        };

        let depth = proto
            .depth
            .ok_or_else(|| prost::DecodeError::new("Missing depth field in DepthEntry"))?;
        let depth = DepthLevel {
            percentage: depth.percentage,
            bid: depth.bid,
            ask: depth.ask,
        };

        let chain = match proto.chain_option {
            Some(crate::schema::depth_entry::ChainOption::NoChain(_)) => None,
            Some(crate::schema::depth_entry::ChainOption::Chain(chain)) => Some(match chain {
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
                    "Missing chain_option field in DepthEntry",
                ))
            }
        };

        Ok(DepthEntry {
            source: proto.source,
            instrument_type,
            pair,
            depth,
            timestamp_ms: proto.timestamp_ms,
            chain,
        })
    }
}

#[cfg(feature = "proto")]
impl crate::ProtoSerialize for DepthEntry {
    fn to_proto_bytes(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto
            .encode(&mut buf)
            .expect("Failed to encode DepthEntry to protobuf");
        buf
    }
}

#[cfg(feature = "proto")]
impl crate::ProtoDeserialize for DepthEntry {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        let proto = crate::schema::DepthEntry::decode(bytes)?;
        Self::from_proto(proto)
    }
}
