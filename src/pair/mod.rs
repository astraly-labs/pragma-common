use std::str::FromStr;

const STABLE_SUFFIXES: [&str; 4] = ["USDT", "USDC", "USD", "DAI"];

/// A pair of assets, e.g. BTC/USD
///
/// This is a simple struct that holds the base and quote assets.
/// It is used to represent a pair of assets in the system.
/// Base and quote are always in UPPERCASE.
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub struct Pair {
    pub base: String,
    pub quote: String,
}

impl Pair {
    /// Creates a routed pair from two pairs that share a common quote currency.
    ///
    /// e.g. "BTC/USD" and "ETH/USD" -> "BTC/ETH"
    pub fn create_routed_pair(base_pair: &Self, quote_pair: &Self) -> Self {
        Self {
            base: base_pair.base.clone(),
            quote: quote_pair.base.clone(),
        }
    }

    /// Creates a new pair from base and quote currencies.
    pub fn from_currencies(base: &str, quote: &str) -> Self {
        Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    /// Creates a pair from a stable pair string with or without delimiters
    /// e.g. "BTCUSDT" -> BTC/USD, "ETH-USDC" -> ETH/USD, "`SOL_USDT`" -> SOL/USD
    pub fn from_stable_pair(pair: &str) -> Option<Self> {
        let pair = pair.to_uppercase();
        let normalized = pair.replace(['-', '_', '/'], "");

        for stable in STABLE_SUFFIXES {
            if let Some(base) = normalized.strip_suffix(stable) {
                return Some(Self {
                    base: base.to_string(),
                    quote: "USD".to_string(),
                });
            }
        }
        None
    }

    /// Get the base and quote as a tuple
    pub fn as_tuple(&self) -> (String, String) {
        (self.base.clone(), self.quote.clone())
    }

    /// Format pair with a custom separator
    pub fn format_with_separator(&self, separator: &str) -> String {
        format!("{}{}{}", self.base, separator, self.quote)
    }

    /// Get the pair ID in standard format without consuming self
    pub fn to_pair_id(&self) -> String {
        self.format_with_separator("/")
    }
}

impl std::fmt::Display for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

impl From<Pair> for String {
    fn from(pair: Pair) -> Self {
        format!("{0}/{1}", pair.base, pair.quote)
    }
}

impl From<&str> for Pair {
    fn from(pair_id: &str) -> Self {
        let normalized = pair_id.replace(['-', '_'], "/");
        let parts: Vec<&str> = normalized.split('/').collect();
        Self {
            base: parts[0].trim().to_uppercase(),
            quote: parts[1].trim().to_uppercase(),
        }
    }
}

impl From<String> for Pair {
    fn from(pair_id: String) -> Self {
        Self::from(pair_id.as_str())
    }
}

impl FromStr for Pair {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl From<(String, String)> for Pair {
    fn from(pair: (String, String)) -> Self {
        Self {
            base: pair.0.to_uppercase(),
            quote: pair.1.to_uppercase(),
        }
    }
}

#[macro_export]
macro_rules! pair {
    ($pair_str:expr) => {{
        #[allow(dead_code)]
        const fn validate_pair(s: &str) -> bool {
            let mut count = 0;
            let chars = s.as_bytes();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == b'/' || chars[i] == b'-' || chars[i] == b'_' {
                    count += 1;
                }
                i += 1;
            }
            count == 1
        }
        const _: () = {
            assert!(
                validate_pair($pair_str),
                "Invalid pair format. Expected format: BASE/QUOTE, BASE-QUOTE, or BASE_QUOTE"
            );
        };
        let normalized = $pair_str.replace('-', "/").replace('_', "/");
        let parts: Vec<&str> = normalized.split('/').collect();
        Pair {
            base: parts[0].trim().to_uppercase(),
            quote: parts[1].trim().to_uppercase(),
        }
    }};
}
