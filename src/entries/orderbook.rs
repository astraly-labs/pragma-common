#[cfg(feature = "proto")]
use prost::Message;

use crate::{instrument_type::InstrumentType, pair::Pair};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct OrderbookEntry {
    pub source: String,
    pub instrument_type: InstrumentType,
    pub pair: Pair,
    pub r#type: OrderbookUpdateType,
    pub data: OrderbookData,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum OrderbookUpdateType {
    Update,
    Snapshot,
}

impl std::fmt::Display for OrderbookUpdateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Update => write!(f, "update"),
            Self::Snapshot => write!(f, "snapshot"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct OrderbookData {
    pub update_id: u64,
    pub bids: Vec<(f64, f64)>,
    pub asks: Vec<(f64, f64)>,
}

#[cfg(feature = "proto")]
impl OrderbookEntry {
    fn to_proto(&self) -> crate::schema::OrderbookEntry {
        crate::schema::OrderbookEntry {
            source: self.source.clone(),
            instrument_type: match self.instrument_type {
                InstrumentType::Spot => crate::schema::InstrumentType::Spot as i32,
                InstrumentType::Perp => crate::schema::InstrumentType::Perp as i32,
            },
            pair: Some(crate::schema::Pair {
                base: self.pair.base.clone(),
                quote: self.pair.quote.clone(),
            }),
            r#type: match self.r#type {
                OrderbookUpdateType::Update => crate::schema::OrderbookUpdateType::Update as i32,
                OrderbookUpdateType::Snapshot => {
                    crate::schema::OrderbookUpdateType::Snapshot as i32
                }
            },
            data: Some(crate::schema::OrderbookData {
                update_id: self.data.update_id,
                bids: self
                    .data
                    .bids
                    .iter()
                    .map(|(price, quantity)| crate::schema::BidOrAsk {
                        price: *price,
                        quantity: *quantity,
                    })
                    .collect(),
                asks: self
                    .data
                    .asks
                    .iter()
                    .map(|(price, quantity)| crate::schema::BidOrAsk {
                        price: *price,
                        quantity: *quantity,
                    })
                    .collect(),
            }),
            timestamp_ms: self.timestamp_ms,
        }
    }

    fn from_proto(proto: crate::schema::OrderbookEntry) -> Result<Self, prost::DecodeError> {
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

        let pair = proto
            .pair
            .ok_or_else(|| prost::DecodeError::new("Missing pair field in OrderbookEntry"))?;
        let pair = Pair {
            base: pair.base,
            quote: pair.quote,
        };

        let r#type = match proto.r#type {
            x if x == crate::schema::OrderbookUpdateType::Update as i32 => {
                OrderbookUpdateType::Update
            }
            x if x == crate::schema::OrderbookUpdateType::Snapshot as i32 => {
                OrderbookUpdateType::Snapshot
            }
            _ => {
                return Err(prost::DecodeError::new(format!(
                    "Invalid type value: {}",
                    proto.r#type,
                )))
            }
        };

        let data = proto
            .data
            .ok_or_else(|| prost::DecodeError::new("Missing data field in OrderbookEntry"))?;
        let bids = data
            .bids
            .iter()
            .map(|bid| (bid.price, bid.quantity))
            .collect();
        let asks = data
            .asks
            .iter()
            .map(|ask| (ask.price, ask.quantity))
            .collect();
        let data = OrderbookData {
            update_id: data.update_id,
            bids,
            asks,
        };

        Ok(OrderbookEntry {
            source: proto.source,
            instrument_type,
            pair,
            r#type,
            data,
            timestamp_ms: proto.timestamp_ms,
        })
    }
}

#[cfg(feature = "proto")]
impl crate::ProtoSerialize for OrderbookEntry {
    fn to_proto_bytes(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto.encode_raw(&mut buf);
        buf
    }
}

#[cfg(feature = "proto")]
impl crate::ProtoDeserialize for OrderbookEntry {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        let proto = crate::schema::OrderbookEntry::decode(bytes)?;
        Self::from_proto(proto)
    }
}
