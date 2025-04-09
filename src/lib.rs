//! Main types used through our rust projects at Pragma.

// Entries, i.e Spot/Perp/Future
pub mod entries;
pub mod instrument;

// Orderbooks, i.e complete orderbooks or snapshot & updates.
pub mod orderbook;

// Telemetry init through OTEL
pub mod telemetry;

// Pair
pub mod pair;

// Starknet on-chain networks
pub mod network;

// Pragma Aggregations
pub mod aggregation;

// Pragma Time Intervals
pub mod interval;
