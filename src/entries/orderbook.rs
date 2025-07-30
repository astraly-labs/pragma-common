#[cfg(feature = "proto")]
use prost::Message;

use crate::{instrument_type::InstrumentType, pair::Pair};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
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
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum OrderbookUpdateType {
    Update(UpdateType),
    Snapshot,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum UpdateType {
    Target,
    Delta,
}

impl std::fmt::Display for UpdateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Target => write!(f, "target"),
            Self::Delta => write!(f, "delta"),
        }
    }
}

impl std::fmt::Display for OrderbookUpdateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Update(update_type) => write!(f, "update with type {}", update_type),
            Self::Snapshot => write!(f, "snapshot"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
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
            r#type: Some(match &self.r#type {
                OrderbookUpdateType::Update(update_type) => crate::schema::OrderbookUpdateType {
                    update_type: Some(crate::schema::orderbook_update_type::UpdateType::Update(
                        match update_type {
                            UpdateType::Target => crate::schema::UpdateType::Target as i32,
                            UpdateType::Delta => crate::schema::UpdateType::Delta as i32,
                        },
                    )),
                },
                OrderbookUpdateType::Snapshot => crate::schema::OrderbookUpdateType {
                    update_type: Some(crate::schema::orderbook_update_type::UpdateType::Snapshot(
                        true,
                    )),
                },
            }),
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
            Some(orderbook_update_type) => match orderbook_update_type.update_type {
                Some(crate::schema::orderbook_update_type::UpdateType::Update(
                    update_type_value,
                )) => {
                    let update_type = match update_type_value {
                        x if x == crate::schema::UpdateType::Target as i32 => UpdateType::Target,
                        x if x == crate::schema::UpdateType::Delta as i32 => UpdateType::Delta,
                        _ => {
                            return Err(prost::DecodeError::new(format!(
                                "Invalid update type value: {}",
                                update_type_value,
                            )))
                        }
                    };
                    OrderbookUpdateType::Update(update_type)
                }
                Some(crate::schema::orderbook_update_type::UpdateType::Snapshot(_)) => {
                    OrderbookUpdateType::Snapshot
                }
                None => {
                    return Err(prost::DecodeError::new(
                        "Missing update_type field in OrderbookUpdateType".to_string(),
                    ))
                }
            },
            None => {
                return Err(prost::DecodeError::new(
                    "Missing type field in OrderbookEntry".to_string(),
                ))
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
