#[cfg(feature = "capnp")]
use pragma_common::{
    entries::{BaseEntry, MarketEntry, SpotEntry},
    orderbook::{Depth, DepthLevel, OrderbookSnapshot, OrderbookUpdate},
    web3::Chain,
    InstrumentType, Pair,
};

#[cfg(feature = "capnp")]
#[test]
fn test_market_entry_capnp() {
    let x = MarketEntry::Spot(SpotEntry {
        base: BaseEntry {
            timestamp: 1,
            source: "TEST".to_string(),
            publisher: "TEST".to_string(),
        },
        pair: Pair::from_currencies("BTC", "USD"),
        price: 12000,
        volume: 0,
    });
    let payload = x.to_capnp();
    let entry: MarketEntry = MarketEntry::from_capnp(&payload).unwrap();
    assert_eq!(entry, x);
}

#[cfg(feature = "capnp")]
#[test]
fn test_depth_capnp() {
    let x = Depth {
        depth: DepthLevel {
            percentage: 0.02,
            bid: 42.69,
            ask: 42.69,
        },
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        source: "TEST".to_string(),
        chain: Some(Chain::Gnosis),
    };
    let payload = x.to_capnp();
    let depth: Depth = Depth::from_capnp(&payload).unwrap();
    assert_eq!(depth, x);
}

#[cfg(feature = "capnp")]
#[test]
fn test_orderbook_update_capnp() {
    let x = OrderbookUpdate {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        last_update_id: 4242,
        bids: vec![(0.0, 1.0), (42.00, 1.0)],
        asks: vec![(42.00, 69.00), (1.00, 42.00)],
    };
    let payload = x.to_capnp();
    let orderbook_update: OrderbookUpdate = OrderbookUpdate::from_capnp(&payload).unwrap();
    assert_eq!(orderbook_update, x);
}

#[cfg(feature = "capnp")]
#[test]
fn test_orderbook_snapshot_capnp() {
    let x = OrderbookSnapshot {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        last_update_id: 4242,
        bids: vec![(0.0, 1.0), (42.00, 1.0)],
        asks: vec![(42.00, 69.00), (1.00, 42.00)],
    };
    let payload = x.to_capnp();
    let orderbook_update: OrderbookSnapshot = OrderbookSnapshot::from_capnp(&payload).unwrap();
    assert_eq!(orderbook_update, x);
}
