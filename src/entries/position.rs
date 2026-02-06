use super::trade::TradeSide;
use crate::{instrument_type::InstrumentType, pair::Pair};
#[cfg(feature = "proto")]
use crate::{ProtoDeserialize, ProtoSerialize};
#[cfg(feature = "proto")]
use prost::Message;
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PositionEntry {
    pub source: String,
    pub instrument_type: InstrumentType,
    pub pair: Pair,
    pub timestamp_ms: i64,
    pub received_timestamp_ms: i64,
    pub side: TradeSide,
    pub notional_in_usd: f64,
    pub size: f64,
}
#[cfg(feature = "proto")]
impl PositionEntry {
    fn to_proto(&self) -> crate::schema::PositionEntry {
        crate::schema::PositionEntry {
            source: self.source.clone(),
            instrument_type: match self.instrument_type {
                InstrumentType::Spot => crate::schema::InstrumentType::Spot as i32,
                InstrumentType::Perp => crate::schema::InstrumentType::Perp as i32,
            },
            pair: Some(crate::schema::Pair {
                base: self.pair.base.clone(),
                quote: self.pair.quote.clone(),
            }),
            timestamp_ms: self.timestamp_ms,
            received_timestamp_ms: self.received_timestamp_ms,
            side: match self.side {
                TradeSide::Buy => crate::schema::TradeSide::Buy as i32,
                TradeSide::Sell => crate::schema::TradeSide::Sell as i32,
            },
            notional_in_usd: self.notional_in_usd,
            size: self.size,
        }
    }
    fn from_proto(proto: crate::schema::PositionEntry) -> Result<Self, prost::DecodeError> {
        let pair = proto
            .pair
            .ok_or_else(|| prost::DecodeError::new("Missing pair field in PositionEntry"))?;

        let instrument_type = match proto.instrument_type {
            x if x == crate::schema::InstrumentType::Spot as i32 => InstrumentType::Spot,
            x if x == crate::schema::InstrumentType::Perp as i32 => InstrumentType::Perp,
            _ => {
                return Err(prost::DecodeError::new(format!(
                    "Invalid instrument_type value: {}",
                    proto.instrument_type,
                )))
            }
        };

        let side = match proto.side {
            x if x == crate::schema::TradeSide::Buy as i32 => TradeSide::Buy,
            x if x == crate::schema::TradeSide::Sell as i32 => TradeSide::Sell,
            _ => {
                return Err(prost::DecodeError::new(format!(
                    "Invalid side value: {}",
                    proto.side,
                )))
            }
        };

        Ok(PositionEntry {
            source: proto.source,
            instrument_type,
            pair: Pair {
                base: pair.base,
                quote: pair.quote,
            },
            timestamp_ms: proto.timestamp_ms,
            received_timestamp_ms: proto.received_timestamp_ms,
            side,
            notional_in_usd: proto.notional_in_usd,
            size: proto.size,
        })
    }
}
#[cfg(feature = "proto")]
impl ProtoSerialize for PositionEntry {
    fn to_proto_bytes(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto
            .encode(&mut buf)
            .expect("Failed to encode PositionEntry to protobuf");
        buf
    }
}
#[cfg(feature = "proto")]
impl ProtoDeserialize for PositionEntry {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        let proto = crate::schema::PositionEntry::decode(bytes)?;
        Self::from_proto(proto)
    }
}
