#[cfg(feature = "proto")]
use pragma_common::{
    entries::funding_rate::FundingRateEntry,
    entries::open_interest::OpenInterestEntry,
    entries::orderbook::{OrderbookData, OrderbookEntry, OrderbookUpdateType, UpdateType},
    entries::price::PriceEntry,
    entries::volume::VolumeEntry,
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
        received_timestamp_ms: 145577,
        instrument_type: InstrumentType::Perp,
    };
    let payload = x.to_proto_bytes();
    let entry: PriceEntry = PriceEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(entry, x);
    assert_eq!(entry.instrument_type, InstrumentType::Perp);
}

#[cfg(feature = "proto")]
#[test]
fn test_orderbook_update_proto() {
    let x = OrderbookEntry {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        r#type: OrderbookUpdateType::Update(UpdateType::Target),
        data: OrderbookData {
            update_id: 4242,
            bids: vec![(0.0, 1.0), (42.00, 1.0)],
            asks: vec![(42.00, 69.00), (1.00, 42.00)],
        },
        timestamp_ms: 145567,
        received_timestamp_ms: 145577,
    };
    let payload = x.to_proto_bytes();
    let orderbook_update: OrderbookEntry = OrderbookEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(orderbook_update, x);
}

#[cfg(feature = "proto")]
#[test]
fn test_orderbook_snapshot_proto() {
    let x = OrderbookEntry {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        r#type: OrderbookUpdateType::Snapshot,
        data: OrderbookData {
            update_id: 4242,
            bids: vec![(0.0, 1.0), (42.00, 1.0)],
            asks: vec![(42.00, 69.00), (1.00, 42.00)],
        },
        timestamp_ms: 145567,
        received_timestamp_ms: 145577,
    };
    let payload = x.to_proto_bytes();
    let orderbook_update: OrderbookEntry = OrderbookEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(orderbook_update, x);
}

#[cfg(feature = "proto")]
#[test]
fn test_orderbook_delta_proto() {
    let x = OrderbookEntry {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        r#type: OrderbookUpdateType::Update(UpdateType::Delta),
        data: OrderbookData {
            update_id: 4242,
            bids: vec![(0.0, 1.0), (42.00, 1.0)],
            asks: vec![(42.00, 69.00), (1.00, 42.00)],
        },
        timestamp_ms: 145567,
        received_timestamp_ms: 145577,
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
        received_timestamp_ms: 145577,
        instrument_type: InstrumentType::Perp,
    };
    let payload = x.to_proto_bytes();
    let entry: FundingRateEntry = FundingRateEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(entry, x);
}

#[cfg(feature = "proto")]
#[test]
fn test_open_interest_entry_proto() {
    let x = OpenInterestEntry {
        source: "TEST".to_string(),
        pair: Pair::from_currencies("BTC", "USD"),
        open_interest: 1000.0,
        timestamp_ms: 145567,
        received_timestamp_ms: 145577,
        instrument_type: InstrumentType::Perp,
    };
    let payload = x.to_proto_bytes();
    let entry: OpenInterestEntry = OpenInterestEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(entry, x);
}

#[cfg(feature = "proto")]
#[test]
fn test_volume_entry_proto() {
    let x = VolumeEntry {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("ETH", "USD"),
        volume_daily: 5000.0,
        timestamp_ms: 145567,
        received_timestamp_ms: 145577,
    };
    let payload = x.to_proto_bytes();
    let entry: VolumeEntry = VolumeEntry::from_proto_bytes(&payload).unwrap();
    assert_eq!(entry, x);
}

#[cfg(feature = "proto")]
#[test]
fn test_trade_entry_proto() {
    use pragma_common::entries::trade::{TradeEntry, TradeSide};

    let x = TradeEntry {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Spot,
        pair: Pair::from_currencies("BTC", "USD"),
        trade_id: "0x4567576".into(),
        buyer_address: "0x1234567890".into(),
        seller_address: "0xabcdef1234567890".into(),
        side: TradeSide::Buy,
        size: 1.0,
        price: 101_024.0,
        timestamp_ms: 145567,
        received_timestamp_ms: 145577,
    };

    let payload = x.to_proto_bytes();
    let entry: TradeEntry = TradeEntry::from_proto_bytes(&payload).unwrap();

    assert_eq!(entry, x);

    let x = TradeEntry {
        source: "TEST".to_string(),
        instrument_type: InstrumentType::Perp,
        pair: Pair::from_currencies("ETH", "USD"),
        trade_id: "0x4567576".into(),
        buyer_address: "0x1234567890".into(),
        seller_address: "0xabcdef1234567890".into(),
        side: TradeSide::Sell,
        size: 1.0,
        price: 101_024.0,
        timestamp_ms: 145567,
        received_timestamp_ms: 145577,
    };

    let payload = x.to_proto_bytes();
    let entry: TradeEntry = TradeEntry::from_proto_bytes(&payload).unwrap();

    assert_eq!(entry, x);
}
