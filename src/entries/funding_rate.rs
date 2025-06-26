#[cfg(feature = "proto")]
use prost::Message;

use crate::Pair;
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
        }
    }

    fn from_proto(proto: crate::schema::FundingRateEntry) -> Result<Self, prost::DecodeError> {
        let pair = proto
            .pair
            .ok_or_else(|| prost::DecodeError::new("Missing pair field in FundingRateEntry"))?;
        Ok(FundingRateEntry {
            source: proto.source,
            pair: Pair {
                base: pair.base,
                quote: pair.quote,
            },
            annualized_rate: proto.annualized_rate,
            timestamp_ms: proto.timestamp_ms,
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
