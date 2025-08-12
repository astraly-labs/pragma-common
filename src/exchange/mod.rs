use crate::{
    pair::{AssetSymbol, RawMarketName},
    Pair,
};

pub mod margin_type;
pub use margin_type::MarginType;

#[derive(
    Clone, Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Copy, strum::EnumString, strum::Display, strum::EnumIter
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[non_exhaustive]
pub enum Exchange {
    Hyperliquid,
    Paradex,
    Kraken,
    Lmax,
    Extended,
}

impl Exchange {
    // TODO: add instrument type argument ?
    /// Returns the market name for the market `pair`
    /// Both base and quote assets are taken into account in the returned market name
    pub fn market_name_from_pair(&self, pair: &Pair) -> RawMarketName {
        match self {
            Exchange::Hyperliquid => pair.base.to_string(),
            Exchange::Paradex => format!("{}-{}-PERP", pair.base, pair.quote),
            Exchange::Kraken => match pair.base.as_str() {
                "BTC" => "PF_XBTUSD".to_string(),
                other => format!("PF_{}{}", other, pair.quote),
            },
            Exchange::Lmax | Exchange::Extended => format!("{}-{}", pair.base, pair.quote),
            _ => unimplemented!("Market name from pair is not supported for this exchange"),
        }
    }

    /// Returns the market name for the market `asset_symbol` with the quote asset being USD
    pub fn usd_market_name_from_asset_symbol(&self, asset_symbol: &AssetSymbol) -> RawMarketName {
        match self {
            Exchange::Hyperliquid => asset_symbol.to_string(),
            Exchange::Paradex => format!("{asset_symbol}-USD-PERP"),
            Exchange::Kraken => match asset_symbol.to_string().to_uppercase().as_str() {
                "BTC" => "PF_XBTUSD".to_string(),
                other => format!("PF_{other}USD"),
            },
            Exchange::Lmax | Exchange::Extended => format!("{}-USD", asset_symbol),
            _ => unimplemented!(
                "USD market name from asset symbol is not supported for this exchange"
            ),
        }
    }

    pub fn asset_symbol_from_raw_market_name(&self, market_name: &RawMarketName) -> AssetSymbol {
        match self {
            Exchange::Hyperliquid => AssetSymbol::from(market_name),
            Exchange::Paradex | Exchange::Lmax | Exchange::Extended => {
                market_name.split('-').next().unwrap().into()
            }
            Exchange::Kraken => {
                if market_name.starts_with("PF_") && market_name.ends_with("USD") {
                    let base_part = &market_name[3..market_name.len() - 3];
                    match base_part {
                        "XBT" => "BTC".into(),
                        other => other.into(),
                    }
                } else {
                    market_name.split('/').next().unwrap().into()
                }
            }
            _ => unimplemented!(
                "Asset symbol from raw market name is not supported for this exchange"
            ),
        }
    }

    /// Returns the taker fees as a percentage
    /// e.g 0.00045 = 0.045%
    pub const fn taker_fees_rate(&self) -> f64 {
        match self {
            // TODO: make this configurable as they have tiers
            Exchange::Hyperliquid => 0.00045, // 0.045% https://hyperliquid.gitbook.io/hyperliquid-docs/trading/fees
            Exchange::Paradex => 0.0003, // 0.03% https://docs.paradex.trade/documentation/trading/trading-fees
            Exchange::Kraken => 0.0002,  // 0.02% https://www.kraken.com/features/fee-schedule
            Exchange::Extended => 0.00025,  // 0.025% https://docs.extended.exchange/extended-resources/trading/trading-fees-and-rebates
            _ => todo!(),
        }
    }

    /// Whether the exchange has some kind of set leverage endpoint
    pub const fn supports_leverage(&self) -> bool {
        match self {
            Exchange::Hyperliquid => true,
            Exchange::Paradex => true,
            Exchange::Kraken => false,
            Exchange::Extended => true,
            Exchange::Lmax => false,
        }
    }

    pub const fn from_str_const(s: &str) -> Option<Self> {
        match s.as_bytes() {
            b"Lmax" | b"lmax" | b"LMAX" => Some(Exchange::Lmax),
            b"Extended" | b"extended" | b"EXTENDED" => Some(Exchange::Extended),
            b"Hyperliquid" | b"hyperliquid" | b"HYPERLIQUID" => Some(Exchange::Hyperliquid),
            b"Paradex" | b"paradex" | b"PARADEX" => Some(Exchange::Paradex),
            b"Kraken" | b"kraken" | b"KRAKEN" => Some(Exchange::Kraken),
            _ => None,
        }
    }
}
