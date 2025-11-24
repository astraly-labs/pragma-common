#[cfg(feature = "proto")]
use prost::Message;

use crate::{instrument_type::InstrumentType, pair::Pair, trading::Side};
#[cfg(feature = "proto")]
use crate::{ProtoDeserialize, ProtoSerialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TradeEntry {
    pub source: String,
    pub instrument_type: InstrumentType,
    pub pair: Pair,
    pub trade_id: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub side: TradeSide,
    pub size: f64,
    pub price: f64,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum TradeSide {
    Buy,
    Sell,
}

impl From<TradeSide> for Side {
    fn from(value: TradeSide) -> Self {
        match value {
            TradeSide::Buy => Self::Long,
            TradeSide::Sell => Self::Short,
        }
    }
}

#[cfg(feature = "proto")]
impl TradeEntry {
    fn to_proto(&self) -> crate::schema::TradeEntry {
        crate::schema::TradeEntry {
            source: self.source.clone(),
            instrument_type: match self.instrument_type {
                InstrumentType::Spot => crate::schema::InstrumentType::Spot as i32,
                InstrumentType::Perp => crate::schema::InstrumentType::Perp as i32,
            },
            pair: Some(crate::schema::Pair {
                base: self.pair.base.clone(),
                quote: self.pair.quote.clone(),
            }),
            buyer_address: self.buyer_address.clone(),
            seller_address: self.seller_address.clone(),
            trade_id: self.trade_id.clone(),
            side: match self.side {
                TradeSide::Buy => crate::schema::TradeSide::Buy as i32,
                TradeSide::Sell => crate::schema::TradeSide::Sell as i32,
            },
            size: self.size,
            price: self.price,
            timestamp_ms: self.timestamp_ms,
        }
    }

    fn from_proto(proto: crate::schema::TradeEntry) -> Result<Self, prost::DecodeError> {
        let pair = proto
            .pair
            .ok_or_else(|| prost::DecodeError::new("Missing pair field in TradeEntry"))?;

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

        Ok(TradeEntry {
            source: proto.source,
            instrument_type,
            pair: Pair {
                base: pair.base,
                quote: pair.quote,
            },
            trade_id: proto.trade_id.clone(),
            buyer_address: proto.buyer_address.clone(),
            seller_address: proto.seller_address.clone(),
            side,
            size: proto.size,
            price: proto.price,
            timestamp_ms: proto.timestamp_ms,
        })
    }
}

#[cfg(feature = "proto")]
impl ProtoSerialize for TradeEntry {
    fn to_proto_bytes(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto
            .encode(&mut buf)
            .expect("Failed to encode VolumeEntry to protobuf");
        buf
    }
}

#[cfg(feature = "proto")]
impl ProtoDeserialize for TradeEntry {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        let proto = crate::schema::TradeEntry::decode(bytes)?;
        Self::from_proto(proto)
    }
}
