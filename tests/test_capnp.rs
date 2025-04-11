#[cfg(feature = "capnp")]
use pragma_common::{
    entries::depth::{DepthEntry, DepthLevel},
    entries::orderbook::{OrderbookData, OrderbookEntry, OrderbookUpdateType},
    entries::price::PriceEntry,
    instrument_type::InstrumentType,
    schema_capnp::{CapnpDeserialize, CapnpSerialize},
    web3::Chain,
    Pair,
};

#[cfg(feature = "capnp")]
#[test]
fn test_price_entry_capnp() {
    let x = PriceEntry {
        source: "TEST".to_string(),
        chain: Some(Chain::Ethereum),
        pair: Pair::from_currencies("BTC", "USD"),
        publisher: "TEST".to_string(),
        timestamp: 145567,
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
    };
    let payload = x.to_capnp();
    let orderbook_update: OrderbookEntry = OrderbookEntry::from_capnp(&payload).unwrap();
    assert_eq!(orderbook_update, x);
}
