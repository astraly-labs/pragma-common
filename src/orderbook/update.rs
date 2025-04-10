#[cfg(feature = "capnp")]
use crate::schema_capnp::{self, orderbook_update};
use crate::{instrument_type::InstrumentType, pair::Pair};
#[cfg(feature = "capnp")]
use capnp::serialize;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub struct OrderbookUpdate {
    pub source: String,
    pub instrument_type: InstrumentType,
    pub pair: Pair,
    pub last_update_id: u64,
    pub bids: Vec<(f64, f64)>,
    pub asks: Vec<(f64, f64)>,
}

#[cfg(feature = "capnp")]
impl OrderbookUpdate {
    pub fn to_capnp(&self) -> Vec<u8> {
        let mut message = capnp::message::Builder::new_default();
        let mut builder = message.init_root::<orderbook_update::Builder>();

        builder.set_source(&self.source);
        builder.set_instrument_type(match self.instrument_type {
            InstrumentType::Spot => schema_capnp::InstrumentType::Spot,
            InstrumentType::Perp => schema_capnp::InstrumentType::Perp,
        });

        let mut pair = builder.reborrow().init_pair();
        pair.set_base(&self.pair.base);
        pair.set_quote(&self.pair.quote);

        builder.set_last_update_id(self.last_update_id);

        let mut bids = builder.reborrow().init_bids(self.bids.len() as u32);
        for (i, &(price, qty)) in self.bids.iter().enumerate() {
            let mut bid = bids.reborrow().get(i as u32);
            bid.set_price(price);
            bid.set_quantity(qty);
        }
        let mut asks = builder.reborrow().init_asks(self.asks.len() as u32);
        for (i, &(price, qty)) in self.asks.iter().enumerate() {
            let mut ask = asks.reborrow().get(i as u32);
            ask.set_price(price);
            ask.set_quantity(qty);
        }

        let mut buffer = Vec::new();
        serialize::write_message(&mut buffer, &message).unwrap();
        buffer
    }

    pub fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error> {
        let message_reader = serialize::read_message(bytes, capnp::message::ReaderOptions::new())?;
        let reader = message_reader.get_root::<orderbook_update::Reader>()?;

        // Extract source
        let source = reader.get_source()?.to_string()?;

        // Extract instrument_type
        let instrument_type = match reader.get_instrument_type()? {
            schema_capnp::InstrumentType::Spot => InstrumentType::Spot,
            schema_capnp::InstrumentType::Perp => InstrumentType::Perp,
        };

        // Extract pair
        let pair_reader = reader.get_pair()?;
        let pair = Pair {
            base: pair_reader.get_base()?.to_string()?,
            quote: pair_reader.get_quote()?.to_string()?,
        };

        // Extract last_update_id
        let last_update_id = reader.get_last_update_id();

        // Extract bids
        let bids_reader = reader.get_bids()?;
        let bids = (0..bids_reader.len())
            .map(|i| {
                let bid = bids_reader.get(i);
                (bid.get_price(), bid.get_quantity())
            })
            .collect::<Vec<(f64, f64)>>();

        // Extract asks
        let asks_reader = reader.get_asks()?;
        let asks = (0..asks_reader.len())
            .map(|i| {
                let ask = asks_reader.get(i);
                (ask.get_price(), ask.get_quantity())
            })
            .collect::<Vec<(f64, f64)>>();

        Ok(OrderbookUpdate {
            source,
            instrument_type,
            pair,
            last_update_id,
            bids,
            asks,
        })
    }
}
