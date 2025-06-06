// Web3 types
pub mod web3;

// Entries retrieved through different markets.
// This is the data that we'll push in our internal Kafka.
pub mod entries;

#[cfg(feature = "starknet")]
pub mod starknet;

// Telemetry init through OTEL
#[cfg(feature = "telemetry")]
pub mod telemetry;

// Trading types
pub mod trading;

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

// Protobuf generated schema. Only related to `entries`.
#[cfg(feature = "proto")]
pub mod schema {
    include!(concat!(env!("OUT_DIR"), "/pragma_common.rs"));
}

// Used to serialize a struct into a payload with protobuf.
#[cfg(feature = "proto")]
pub trait ProtoSerialize {
    fn to_proto_bytes(&self) -> Vec<u8>;
}

// Used to deserialize a protobuf payload into a struct.
#[cfg(feature = "proto")]
pub trait ProtoDeserialize {
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError>
    where
        Self: Sized;
}
