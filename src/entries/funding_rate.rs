#[cfg(feature = "proto")]
use prost::Message;

use crate::{instrument_type::InstrumentType, Pair};
#[cfg(feature = "proto")]
use crate::{ProtoDeserialize, ProtoSerialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FundingRateEntry {
    pub source: String,
    pub pair: Pair,
    pub annualized_rate: f64,
    pub timestamp_ms: i64,
    pub instrument_type: InstrumentType,
    pub received_timestamp_ms: i64,
}

#[cfg(feature = "proto")]
impl FundingRateEntry {
    fn to_proto(&self) -> crate::schema::FundingRateEntry {
        crate::schema::FundingRateEntry {
            source: self.source.clone(),
            pair: Some(crate::schema::Pair {
                base: self.pair.base.clone(),
                quote: self.pair.quote.clone(),
            }),
            annualized_rate: self.annualized_rate,
            timestamp_ms: self.timestamp_ms,
            instrument_type: match self.instrument_type {
                InstrumentType::Spot => crate::schema::InstrumentType::Spot as i32,
                InstrumentType::Perp => crate::schema::InstrumentType::Perp as i32,
            },
            received_timestamp_ms: self.received_timestamp_ms,
        }
    }

    fn from_proto(proto: crate::schema::FundingRateEntry) -> Result<Self, prost::DecodeError> {
        let pair = proto
            .pair
            .ok_or_else(|| prost::DecodeError::new("Missing pair field in FundingRateEntry"))?;
        let instrument_type = match proto.instrument_type {
            x if x == crate::schema::InstrumentType::Spot as i32 => InstrumentType::Spot,
            x if x == crate::schema::InstrumentType::Perp as i32 => InstrumentType::Perp,
            _ => InstrumentType::Perp, // Default to Perp for funding rates (backwards compat)
        };
        Ok(FundingRateEntry {
            source: proto.source,
            pair: Pair {
                base: pair.base,
                quote: pair.quote,
            },
            annualized_rate: proto.annualized_rate,
            timestamp_ms: proto.timestamp_ms,
            instrument_type,
            received_timestamp_ms: proto.received_timestamp_ms,
        })
    }
}

#[cfg(feature = "proto")]
impl ProtoSerialize for FundingRateEntry {
    fn to_proto_bytes(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto.encode_raw(&mut buf);
        buf
    }
}

#[cfg(feature = "proto")]
impl ProtoDeserialize for FundingRateEntry {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        let proto = crate::schema::FundingRateEntry::decode(bytes)?;
        Self::from_proto(proto)
    }
}
