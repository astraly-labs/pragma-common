@0xf77e83e1f4aad73b;

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

enum OrderbookUpdateType {
  update @0;
  snapshot @1;
}

# Structs for custom types
struct Pair {
  base @0 :Text;
  quote @1 :Text;
}

struct BidOrAsk {
  price @0 :Float64;
  quantity @1 :Float64;
}

struct DepthLevel {
  percentage @0 :Float64;
  bid @1 :Float64;
  ask @2 :Float64;
}

struct OrderbookData {
  updateId @0 :UInt64;
  bids @1 :List(BidOrAsk);
  asks @2 :List(BidOrAsk);
}

# Main structs
struct PriceEntry {
  source @0 :Text;
  chain: union {
    noChain @1 :Void;
    chain @2 :Chain;
  }
  pair @3 :Pair;
  timestamp @4 :Int64;
  price @5 :UInt128;
  volume @6 :UInt128;
  expirationTimestamp :union {
    noExpiration @7 :Void;
    expirationTimestamp @8 :Int64;
  }
}

struct OrderbookEntry {
  source @0 :Text;
  instrumentType @1 :InstrumentType;
  pair @2 :Pair;
  type @3 :OrderbookUpdateType;
  data @4 :OrderbookData;
  timestamp @5 :Int64;
}

struct DepthEntry {
  source @0 :Text;
  instrumentType @1 :InstrumentType;
  pair @2 :Pair;
  depth @3 :DepthLevel;
  chain: union {
    noChain @4 :Void;
    chain @5 :Chain;
  }
  timestamp @6 :Int64;
}