#[cfg(feature = "proto")]
use pragma_common::{
    entries::depth::{DepthEntry, DepthLevel},
    entries::funding_rate::FundingRateEntry,
    entries::orderbook::{OrderbookData, OrderbookEntry, OrderbookUpdateType},
    entries::price::PriceEntry,
    instrument_type::InstrumentType,
    web3::Chain,
    Pair, ProtoDeserialize, ProtoSerialize,
};

#[cfg(feature = "proto")]
#[test]
fn test_price_entry_proto() {
    let x = PriceEntry {
        source: "TEST".to_string(),
        chain: Some(Chain::Ethereum),
        pair: Pair::from_currencies("BTC", "USD"),
        timestamp_ms: 145567,
        price: 12000,
        volume: 0,
        expiration_timestamp: Some(0),
    };
    let payload = x.to_proto_bytes();
    let entry: PriceEntry = PriceEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(entry, x);
    assert_eq!(entry.instrument_type(), InstrumentType::Perp);
}

#[cfg(feature = "proto")]
#[test]
fn test_depth_entry_proto() {
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
    let payload = x.to_proto_bytes();
    let depth: DepthEntry = DepthEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(depth, x);
}

#[cfg(feature = "proto")]
#[test]
fn test_orderbook_update_proto() {
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
    let payload = x.to_proto_bytes();
    let orderbook_update: OrderbookEntry = OrderbookEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(orderbook_update, x);
}

#[cfg(feature = "proto")]
#[test]
fn test_annualized_rate_proto() {
    let x = FundingRateEntry {
        source: "TEST".to_string(),
        pair: Pair::from_currencies("BTC", "USD"),
        annualized_rate: 42.42,
        timestamp_ms: 145567,
    };
    let payload = x.to_proto_bytes();
    let entry: FundingRateEntry = FundingRateEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(entry, x);
}
