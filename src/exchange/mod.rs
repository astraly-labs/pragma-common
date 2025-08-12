use crate::{pair::AssetSymbol, Pair};

pub mod currency;
pub mod market;

#[derive(
    Clone, Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Copy, strum::EnumString, strum::Display,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum MarginType {
    Isolated,
    Cross,
}

#[derive(
    Clone, Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Copy, strum::EnumString, strum::Display,
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
    // TODO: add instrument type to the market name
    pub fn market_name_from_pair(&self, pair: &Pair) -> String {
        match self {
            Exchange::Hyperliquid => pair.base.to_string(),
            Exchange::Paradex => format!("{}-USD-PERP", pair.base),
            Exchange::Kraken => match pair.base.as_str() {
                "BTC" => "PF_XBTUSD".to_string(),
                other => format!("PF_{other}USD"),
            },
            _ => todo!(),
        }
    }

    pub fn market_name_from_asset_symbol(&self, asset_symbol: &AssetSymbol) -> String {
        match self {
            Exchange::Hyperliquid => asset_symbol.to_string(),
            Exchange::Paradex => format!("{asset_symbol}-USD-PERP"),
            Exchange::Kraken => match asset_symbol.to_string().to_uppercase().as_str() {
                "BTC" => "PF_XBTUSD".to_string(),
                other => format!("PF_{other}USD"),
            },
            _ => todo!(),
        }
    }

    pub fn asset_symbol_from_raw_market_name(&self, market_name: &str) -> AssetSymbol {
        match self {
            Exchange::Hyperliquid => AssetSymbol::from(market_name),
            Exchange::Paradex => market_name.split('-').next().unwrap().into(),
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
            _ => todo!(),
        }
    }

    pub fn fee_rate(&self) -> f64 {
        match self {
            // TODO: make this configurable as they have tiers
            Exchange::Hyperliquid => 0.00045, // 0.045% https://hyperliquid.gitbook.io/hyperliquid-docs/trading/fees
            Exchange::Paradex => 0.0003, // 0.03% https://docs.paradex.trade/documentation/trading/trading-fees
            Exchange::Kraken => 0.0002,  // 0.02% https://www.kraken.com/features/fee-schedule
            _ => todo!(),
        }
    }

    /// Whether the exchange has some kind of set leverage endpoint
    pub fn supports_leverage(&self) -> bool {
        match self {
            Exchange::Hyperliquid => true,
            Exchange::Paradex => true,
            Exchange::Kraken => false,
            _ => todo!(),
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
