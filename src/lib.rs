//! pragma-common
//! Main types used through our rust projects at Pragma.
// Entries retrieved through different markets.
pub mod entries;

// Web3 types
pub mod web3;

// Telemetry init through OTEL
#[cfg(feature = "telemetry")]
pub mod telemetry;

// Pair
pub mod pair;
pub use pair::Pair;

// A complete managed Orderbook.
pub mod orderbook;
pub use orderbook::{Orderbook, OrderbookError};

// Types of instrument supported, i.e spot, perp etc.
pub mod instrument_type;
pub use instrument_type::{InstrumentType, InstrumentTypeError};

// Pragma Aggregations
pub mod aggregation;
pub use aggregation::AggregationMode;

// Pragma Time Intervals
pub mod interval;
pub use interval::Interval;

// Capnp generated schema
#[cfg(feature = "capnp")]
mod schema_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema_capnp.rs"));
}

#[cfg(feature = "capnp")]
pub trait CapnpSerialize {
    fn to_capnp(&self) -> Vec<u8>;
}

#[cfg(feature = "capnp")]
pub trait CapnpDeserialize {
    fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error>
    where
        Self: Sized;
}
