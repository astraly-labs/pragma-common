#[cfg(feature = "proto")]
use prost::Message;

use crate::{InstrumentType, Pair};
#[cfg(feature = "proto")]
use crate::{ProtoDeserialize, ProtoSerialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct OpenInterestEntry {
    pub source: String,
    pub instrument_type: InstrumentType,
    pub pair: Pair,
    pub open_interest: f64,
    pub timestamp_ms: i64,
}

#[cfg(feature = "proto")]
impl OpenInterestEntry {
    fn to_proto(&self) -> crate::schema::OpenInterestEntry {
        crate::schema::OpenInterestEntry {
            source: self.source.clone(),
            instrument_type: match self.instrument_type {
                InstrumentType::Spot => crate::schema::InstrumentType::Spot as i32,
                InstrumentType::Perp => crate::schema::InstrumentType::Perp as i32,
            },
            pair: Some(crate::schema::Pair {
                base: self.pair.base.clone(),
                quote: self.pair.quote.clone(),
            }),
            open_interest: self.open_interest,
            timestamp_ms: self.timestamp_ms,
        }
    }

    fn from_proto(proto: crate::schema::OpenInterestEntry) -> Result<Self, prost::DecodeError> {
        let pair = proto
            .pair
            .ok_or_else(|| prost::DecodeError::new("Missing pair field in OpenInterestEntry"))?;

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

        Ok(OpenInterestEntry {
            source: proto.source,
            instrument_type,
            pair: Pair {
                base: pair.base,
                quote: pair.quote,
            },
            open_interest: proto.open_interest,
            timestamp_ms: proto.timestamp_ms,
        })
    }
}

#[cfg(feature = "proto")]
impl ProtoSerialize for OpenInterestEntry {
    fn to_proto_bytes(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto
            .encode(&mut buf)
            .expect("Failed to encode OpenInterestEntry to protobuf");
        buf
    }
}

#[cfg(feature = "proto")]
impl ProtoDeserialize for OpenInterestEntry {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        let proto = crate::schema::OpenInterestEntry::decode(bytes)?;
        Self::from_proto(proto)
    }
}
