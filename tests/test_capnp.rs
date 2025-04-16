#[cfg(feature = "capnp")]
use pragma_common::{
    entries::depth::{DepthEntry, DepthLevel},
    entries::funding_rate::FundingRateEntry,
    entries::orderbook::{OrderbookData, OrderbookEntry, OrderbookUpdateType},
    entries::price::PriceEntry,
    instrument_type::InstrumentType,
    web3::Chain,
    CapnpDeserialize, CapnpSerialize, Pair,
};

#[cfg(feature = "capnp")]
#[test]
fn test_price_entry_capnp() {
    let x = PriceEntry {
        source: "TEST".to_string(),
        chain: Some(Chain::Ethereum),
        pair: Pair::from_currencies("BTC", "USD"),
        timestamp_ms: 145567,
        price: 12000,
        volume: 0,
        expiration_timestamp: Some(0),
    };
    let payload = x.to_capnp();
    let entry: PriceEntry = PriceEntry::from_capnp(&payload).unwrap();
    assert_eq!(entry, x);
    assert_eq!(entry.instrument_type(), InstrumentType::Perp);
}

#[cfg(feature = "capnp")]
#[test]
fn test_depth_entry_capnp() {
    let x = DepthEntry {
        depth: DepthLevel {
            percentage: 0.02,
            bid: 42.69,
            ask: 42.69,
        },
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        source: "TEST".to_string(),
        chain: Some(Chain::Gnosis),
        timestamp_ms: 145567,
    };
    let payload = x.to_capnp();
    let depth: DepthEntry = DepthEntry::from_capnp(&payload).unwrap();
    assert_eq!(depth, x);
}

#[cfg(feature = "capnp")]
#[test]
fn test_orderbook_update_capnp() {
    let x = OrderbookEntry {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        r#type: OrderbookUpdateType::Update,
        data: OrderbookData {
            update_id: 4242,
            bids: vec![(0.0, 1.0), (42.00, 1.0)],
            asks: vec![(42.00, 69.00), (1.00, 42.00)],
        },
        timestamp_ms: 145567,
    };
    let payload = x.to_capnp();
    let orderbook_update: OrderbookEntry = OrderbookEntry::from_capnp(&payload).unwrap();
    assert_eq!(orderbook_update, x);
}

#[cfg(feature = "capnp")]
#[test]
fn test_funding_rate_capnp() {
    let x = FundingRateEntry {
        source: "TEST".to_string(),
        pair: Pair::from_currencies("BTC", "USD"),
        funding_rate: 42.42,
        timestamp_ms: 145567,
    };
    let payload = x.to_capnp();
    let entry: FundingRateEntry = FundingRateEntry::from_capnp(&payload).unwrap();
    assert_eq!(entry, x);
}
