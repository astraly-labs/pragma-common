pub mod base;
pub mod perp;
pub mod spot;

pub use base::*;
pub use perp::*;
pub use spot::*;

#[cfg(feature = "capnp")]
use crate::schema_capnp::{self, market_entry};
use crate::{instrument_type::InstrumentType, pair::Pair};
#[cfg(feature = "capnp")]
use capnp::serialize;

pub trait EntryTrait {
    fn base(&self) -> &base::BaseEntry;
    fn pair(&self) -> &Pair;
    fn price(&self) -> u128;
    fn volume(&self) -> u128;
    fn expiration_timestamp_ms(&self) -> Option<u64>;
    fn instrument_type(&self) -> InstrumentType;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
pub enum MarketEntry {
    #[cfg_attr(feature = "serde", serde(rename = "spot"))]
    Spot(spot::SpotEntry),
    #[cfg_attr(feature = "serde", serde(rename = "perp"))]
    Perp(perp::PerpEntry),
}

impl EntryTrait for MarketEntry {
    fn instrument_type(&self) -> InstrumentType {
        match self {
            Self::Spot(entry) => entry.instrument_type(),
            Self::Perp(entry) => entry.instrument_type(),
        }
    }

    fn base(&self) -> &base::BaseEntry {
        match self {
            Self::Spot(entry) => entry.base(),
            Self::Perp(entry) => entry.base(),
        }
    }

    fn pair(&self) -> &Pair {
        match self {
            Self::Spot(entry) => entry.pair(),
            Self::Perp(entry) => entry.pair(),
        }
    }

    fn price(&self) -> u128 {
        match self {
            Self::Spot(entry) => entry.price(),
            Self::Perp(entry) => entry.price(),
        }
    }

    fn volume(&self) -> u128 {
        match self {
            Self::Spot(entry) => entry.volume(),
            Self::Perp(entry) => entry.volume(),
        }
    }

    fn expiration_timestamp_ms(&self) -> Option<u64> {
        match self {
            Self::Spot(entry) => entry.expiration_timestamp_ms(),
            Self::Perp(entry) => entry.expiration_timestamp_ms(),
        }
    }
}

impl std::fmt::Display for MarketEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spot(entry) => write!(f, "spot: {entry}"),
            Self::Perp(entry) => write!(f, "perp: {entry}"),
        }
    }
}

impl From<MarketEntry> for InstrumentType {
    fn from(value: MarketEntry) -> Self {
        value.instrument_type()
    }
}

#[cfg(feature = "capnp")]
impl MarketEntry {
    pub fn to_capnp(&self) -> Vec<u8> {
        let mut message = capnp::message::Builder::new_default();
        let mut builder = message.init_root::<market_entry::Builder>();

        // Set base (assuming BaseEntry has a timestamp)
        let mut base = builder.reborrow().init_base();
        base.set_timestamp(self.base().timestamp);
        base.set_source(self.base().source.clone());
        base.set_publisher(self.base().publisher.clone());

        // Set pair
        let mut pair = builder.reborrow().init_pair();
        pair.set_base(&self.pair().base);
        pair.set_quote(&self.pair().quote);

        // Set price and volume (u128 to UInt128)
        let price = self.price();
        let volume = self.volume();
        let mut price_builder = builder.reborrow().init_price();
        price_builder.set_low(price as u64);
        price_builder.set_high((price >> 64) as u64);
        let mut volume_builder = builder.reborrow().init_volume();
        volume_builder.set_low(volume as u64);
        volume_builder.set_high((volume >> 64) as u64);

        // Set instrument_type
        builder.set_instrument_type(match self.instrument_type() {
            InstrumentType::Spot => schema_capnp::InstrumentType::Spot,
            InstrumentType::Perp => schema_capnp::InstrumentType::Perp,
        });

        // Set expiration_timestamp_ms
        match self.expiration_timestamp_ms() {
            Some(ts) => builder.reborrow().set_expiration_timestamp_ms(ts),
            None => builder.reborrow().set_no_expiration(()),
        };

        let mut buffer = Vec::new();
        serialize::write_message(&mut buffer, &message).unwrap();
        buffer
    }

    pub fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error> {
        // Read the byte payload into a Cap'n Proto message reader
        let message_reader = serialize::read_message(bytes, capnp::message::ReaderOptions::new())?;
        let reader = message_reader.get_root::<market_entry::Reader>()?;

        // Extract base
        let base_reader = reader.get_base()?;
        let base = base::BaseEntry {
            timestamp: base_reader.get_timestamp(),
            source: base_reader.get_source()?.to_string()?,
            publisher: base_reader.get_publisher()?.to_string()?,
        };

        // Extract pair
        let pair_reader = reader.get_pair()?;
        let pair = Pair {
            base: pair_reader.get_base()?.to_string()?,
            quote: pair_reader.get_quote()?.to_string()?,
        };

        // Extract price and volume (UInt128 to u128)
        let price_reader = reader.get_price()?;
        let price = (price_reader.get_high() as u128) << 64 | price_reader.get_low() as u128;
        let volume_reader = reader.get_volume()?;
        let volume = (volume_reader.get_high() as u128) << 64 | volume_reader.get_low() as u128;

        // Extract instrument_type
        let instrument_type = match reader.get_instrument_type()? {
            schema_capnp::InstrumentType::Spot => InstrumentType::Spot,
            schema_capnp::InstrumentType::Perp => InstrumentType::Perp,
        };

        // Construct the appropriate MarketEntry variant
        let entry = match instrument_type {
            InstrumentType::Spot => MarketEntry::Spot(spot::SpotEntry {
                base,
                pair,
                price,
                volume,
            }),
            InstrumentType::Perp => MarketEntry::Perp(perp::PerpEntry {
                base,
                pair,
                price,
                volume,
            }),
        };

        Ok(entry)
    }
}
