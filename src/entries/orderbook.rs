#[cfg(feature = "capnp")]
use crate::schema_capnp;
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

#[cfg(feature = "capnp")]
impl crate::CapnpSerialize for OrderbookEntry {
    fn to_capnp(&self) -> Vec<u8> {
        // Initialize a new Cap'n Proto message builder
        let mut message = capnp::message::Builder::new_default();
        let mut builder = message.init_root::<schema_capnp::orderbook_entry::Builder>();

        // Set the source field
        builder.set_source(&self.source);

        // Set the instrument_type field by mapping Rust enum to Cap'n Proto enum
        builder.set_instrument_type(match self.instrument_type {
            InstrumentType::Spot => schema_capnp::InstrumentType::Spot,
            InstrumentType::Perp => schema_capnp::InstrumentType::Perp,
        });

        // Set the pair field
        let mut pair = builder.reborrow().init_pair();
        pair.set_base(&self.pair.base);
        pair.set_quote(&self.pair.quote);

        // Set the type field by mapping Rust enum to Cap'n Proto enum
        builder.set_type(match self.r#type {
            OrderbookUpdateType::Update => schema_capnp::OrderbookUpdateType::Update,
            OrderbookUpdateType::Snapshot => schema_capnp::OrderbookUpdateType::Snapshot,
        });

        // Set the data field (OrderbookData)
        let mut data = builder.reborrow().init_data();
        data.set_update_id(self.data.update_id);

        // Set the bids list
        let mut bids = data.reborrow().init_bids(self.data.bids.len() as u32);
        for (i, (price, quantity)) in self.data.bids.iter().enumerate() {
            let mut bid = bids.reborrow().get(i as u32);
            bid.set_price(*price);
            bid.set_quantity(*quantity);
        }

        // Set the asks list
        let mut asks = data.reborrow().init_asks(self.data.asks.len() as u32);
        for (i, (price, quantity)) in self.data.asks.iter().enumerate() {
            let mut ask = asks.reborrow().get(i as u32);
            ask.set_price(*price);
            ask.set_quantity(*quantity);
        }

        // Set timestamp_ms
        builder.set_timestamp_ms(self.timestamp_ms);

        // Serialize the message to a byte vector
        let mut buffer = Vec::new();
        capnp::serialize::write_message(&mut buffer, &message).unwrap();
        buffer
    }
}

#[cfg(feature = "capnp")]
impl crate::CapnpDeserialize for OrderbookEntry {
    fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error>
    where
        Self: Sized,
    {
        // Read the Cap'n Proto message
        let message_reader =
            capnp::serialize::read_message(bytes, capnp::message::ReaderOptions::new())?;
        let reader = message_reader.get_root::<schema_capnp::orderbook_entry::Reader>()?;

        // Extract the source field
        let source = reader.get_source()?.to_string()?;

        // Extract the instrument_type field
        let instrument_type = match reader.get_instrument_type()? {
            schema_capnp::InstrumentType::Spot => InstrumentType::Spot,
            schema_capnp::InstrumentType::Perp => InstrumentType::Perp,
        };

        // Extract the pair field
        let pair_reader = reader.get_pair()?;
        let pair = Pair {
            base: pair_reader.get_base()?.to_string()?,
            quote: pair_reader.get_quote()?.to_string()?,
        };

        // Extract the type field
        let r#type = match reader.get_type()? {
            schema_capnp::OrderbookUpdateType::Update => OrderbookUpdateType::Update,
            schema_capnp::OrderbookUpdateType::Snapshot => OrderbookUpdateType::Snapshot,
        };

        // Extract the data field (OrderbookData)
        let data_reader = reader.get_data()?;
        let update_id = data_reader.get_update_id();

        // Extract the bids list
        let bids_reader = data_reader.get_bids()?;
        let bids = (0..bids_reader.len())
            .map(|i| {
                let bid = bids_reader.get(i);
                Ok((bid.get_price(), bid.get_quantity()))
            })
            .collect::<Result<Vec<_>, capnp::Error>>()?;

        // Extract the asks list
        let asks_reader = data_reader.get_asks()?;
        let asks = (0..asks_reader.len())
            .map(|i| {
                let ask = asks_reader.get(i);
                Ok((ask.get_price(), ask.get_quantity()))
            })
            .collect::<Result<Vec<_>, capnp::Error>>()?;

        // Extract timestamp_ms
        let timestamp_ms = reader.get_timestamp_ms();

        // Construct the OrderbookData struct
        let data = OrderbookData {
            update_id,
            bids,
            asks,
        };

        // Construct and return the OrderbookEntry
        Ok(OrderbookEntry {
            source,
            instrument_type,
            pair,
            r#type,
            data,
            timestamp_ms,
        })
    }
}
