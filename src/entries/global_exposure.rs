#[cfg(feature = "proto")]
use prost::Message;

use crate::contract::Contract;
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
    pub gross_position_size: f64,
    pub net_position_size: f64,
    pub contract: Option<Contract>,
}

#[cfg(feature = "proto")]
impl GlobalExposureEntry {
    fn to_proto(&self) -> crate::schema::GlobalExposureEntry {
        crate::schema::GlobalExposureEntry {
            source: self.source.clone(),
            timestamp_ms: self.timestamp_ms,
            asset: self.asset.clone(),
            gross_position_size: self.gross_position_size,
            net_position_size: self.net_position_size,
            contract: self.contract.map(Contract::to_proto),
        }
    }

    fn from_proto(proto: crate::schema::GlobalExposureEntry) -> Result<Self, prost::DecodeError> {
        Ok(GlobalExposureEntry {
            source: proto.source,
            timestamp_ms: proto.timestamp_ms,
            asset: proto.asset,
            gross_position_size: proto.gross_position_size,
            net_position_size: proto.net_position_size,
            contract: proto.contract.map(Contract::from_proto).transpose()?,
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
        let proto = crate::schema::GlobalExposureEntry::decode(bytes)?;
        Self::from_proto(proto)
    }
}

#[cfg(all(test, feature = "proto"))]
mod tests {
    use super::*;

    #[test]
    fn proto_roundtrip_preserves_contract() {
        let entry = GlobalExposureEntry {
            source: "EXPOSURE_AGGREGATOR".to_string(),
            timestamp_ms: 1,
            asset: "WTI".to_string(),
            gross_position_size: 10.0,
            net_position_size: -4.0,
            contract: Some(Contract::from_raw_symbol("CLK6").unwrap()),
        };

        let decoded = GlobalExposureEntry::from_proto_bytes(&entry.to_proto_bytes()).unwrap();

        assert_eq!(decoded, entry);
    }
}
