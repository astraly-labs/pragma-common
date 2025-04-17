//! pragma-common
//! Main types used through our rust projects at Pragma.
// Web3 types
pub mod web3;

// Entries retrieved through different markets.
// This is the data that we'll push in our internal Kafka.
pub mod entries;

// Telemetry init through OTEL
#[cfg(feature = "telemetry")]
pub mod telemetry;

// Pair
pub mod pair;
pub use pair::Pair;

// Types of instrument supported, i.e spot, perp etc.
pub mod instrument_type;
pub use instrument_type::{InstrumentType, InstrumentTypeError};

// Pragma Aggregations
pub mod aggregation;
pub use aggregation::AggregationMode;

// An util to manage multiple tasks gracefully
#[cfg(feature = "services")]
pub mod services;

// A structure allowing us to have multiple handles dependent on each others.
#[cfg(feature = "task-group")]
pub mod task_group;

// Pragma Time Intervals
pub mod interval;
pub use interval::Interval;

// Capnp generated schema. Only related to `entries`.
#[cfg(feature = "capnp")]
mod schema_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema_capnp.rs"));
}

// Used to serialize a struct into a payload with capnp.
#[cfg(feature = "capnp")]
pub trait CapnpSerialize {
    fn to_capnp(&self) -> Vec<u8>;
}

// Used to deserialize a capnp payload into a struct.
#[cfg(feature = "capnp")]
pub trait CapnpDeserialize {
    fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error>
    where
        Self: Sized;
}
