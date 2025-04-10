//! Main types used through our rust projects at Pragma.

// Entries, i.e Spot/Perp/Future
pub mod entries;
pub mod instrument_type;
pub use instrument_type::InstrumentType;

// Web3 types
pub mod web3;

// Orderbooks, i.e complete orderbooks or snapshot & updates.
pub mod orderbook;

// Telemetry init through OTEL
#[cfg(feature = "telemetry")]
pub mod telemetry;

// Pair
pub mod pair;
pub use pair::Pair;

// Pragma Aggregations
pub mod aggregation;
pub use aggregation::AggregationMode;

// Pragma Time Intervals
pub mod interval;
pub use interval::Interval;

// Capnp generated schema
#[cfg(feature = "capnp")]
pub mod schema_capnp {
    pub trait CapnpSerialize {
        fn to_capnp(&self) -> Vec<u8>;
    }

    pub trait CapnpDeserialize {
        fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error>
        where
            Self: Sized;
    }

    include!(concat!(env!("OUT_DIR"), "/schema_capnp.rs"));
}
