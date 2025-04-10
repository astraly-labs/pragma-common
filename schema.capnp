@0x986b3393db1396c9;  # Unique file ID

# Utils types
struct UInt128 {
  low @0 :UInt64;
  high @1 :UInt64;
}

# Enums
enum InstrumentType {
  spot @0;
  perp @1;
}

enum Chain {
  starknet @0;
  solana @1;
  sui @2;
  aptos @3;
  ethereum @4;
  base @5;
  arbitrum @6;
  optimism @7;
  zksync @8;
  polygon @9;
  bnb @10;
  avalanche @11;
  gnosis @12;
  worldchain @13;
}

# Structs for custom types
struct Pair {
  base @0 :Text;
  quote @1 :Text;
}

struct BaseEntry {
  timestamp @0 :Int64;
  source @1 :Text;
  publisher @2 :Text;
}

# Same structure for bid & ask, basically just a price & a quantity
struct BidOrAsk {
  price @0 :Float64;
  quantity @1 :Float64;
}

struct DepthLevel {
  percentage @0 :Float64;
  bid @1 :Float64;
  ask @2 :Float64;
}

# Main structs
struct MarketEntry {
  base @0 :BaseEntry;
  pair @1 :Pair;
  price @2 :UInt128;
  volume @3 :UInt128;
  instrumentType @4 :InstrumentType;
  union {
    noExpiration @5 :Void;         # Represents Option<u64>
    expirationTimestampMs @6 :UInt64;
  }
}

struct OrderbookSnapshot {
  source @0 :Text;
  instrumentType @1 :InstrumentType;
  pair @2 :Pair;
  lastUpdateId @3 :UInt64;
  bids @4 :List(BidOrAsk);
  asks @5 :List(BidOrAsk);
}

struct OrderbookUpdate {
  source @0 :Text;
  instrumentType @1 :InstrumentType;
  pair @2 :Pair;
  lastUpdateId @3 :UInt64;
  bids @4 :List(BidOrAsk);
  asks @5 :List(BidOrAsk);
}

struct Depth {
  depth @0 :DepthLevel;
  pair @1 :Pair;
  source @2 :Text;
  instrumentType @3 :InstrumentType;
  union {
    noChain @4 :Void;    # Represents Option<Chain>
    chain @5 :Chain;
  }
}