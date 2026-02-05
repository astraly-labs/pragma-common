#[cfg(feature = "proto")]
use prost::Message;

#[cfg(feature = "proto")]
use crate::{ProtoDeserialize, ProtoSerialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GlobalExposureEntry {
    pub source: String,
    pub timestamp_ms: i64,
    pub asset: String,
    pub exposure: f64,
}

#[cfg(feature = "proto")]
impl GlobalExposureEntry {
    fn to_proto(&self) -> crate::schema::GlobalExposure {
        crate::schema::GlobalExposure {
            source: self.source.clone(),
            timestamp_ms: self.timestamp_ms,
            asset: self.asset.clone(),
            exposure: self.exposure,
        }
    }

    fn from_proto(proto: crate::schema::GlobalExposure) -> Result<Self, prost::DecodeError> {
        Ok(GlobalExposureEntry {
            source: proto.source,
            timestamp_ms: proto.timestamp_ms,
            asset: proto.asset,
            exposure: proto.exposure,
        })
    }
}

#[cfg(feature = "proto")]
impl ProtoSerialize for GlobalExposureEntry {
    fn to_proto_bytes(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto
            .encode(&mut buf)
            .expect("Failed to encode GlobalExposureEntry to protobuf");
        buf
    }
}

#[cfg(feature = "proto")]
impl ProtoDeserialize for GlobalExposureEntry {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        let proto = crate::schema::GlobalExposure::decode(bytes)?;
        Self::from_proto(proto)
    }
}

