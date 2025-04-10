//! Main types used through our rust projects at Pragma.

// Entries, i.e Spot/Perp/Future
pub mod entries;
pub mod instrument;
pub use instrument::Instrument;

// Web3 types
pub mod web3;

// Orderbooks, i.e complete orderbooks or snapshot & updates.
pub mod orderbook;

// Telemetry init through OTEL
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
