use std::str::FromStr;

const STABLE_SUFFIXES: [&str; 4] = ["USDT", "USDC", "USD", "DAI"];

pub type AssetSymbol = String;
pub type RawMarketName = String;

/// A pair of assets, e.g. BTC/USD
///
/// This is a simple struct that holds the base and quote assets.
/// It is used to represent a pair of assets in the system.
/// Base and quote are always in UPPERCASE.
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Pair {
    pub base: AssetSymbol,
    pub quote: AssetSymbol,
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
    pub fn as_tuple(&self) -> (AssetSymbol, AssetSymbol) {
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

impl TryFrom<&str> for Pair {
    type Error = anyhow::Error;

    fn try_from(pair_id: &str) -> anyhow::Result<Self> {
        // Normalize: replace "-" and "_" with "/"
        let normalized = pair_id.replace(['-', '_'], "/");

        // Split into parts
        let parts: Vec<&str> = normalized.split('/').collect();

        // Validate: exactly 2 parts
        if parts.len() != 2 || parts[0].trim().is_empty() || parts[1].trim().is_empty() {
            anyhow::bail!("Invalid pair format: expected format like A/B");
        }

        Ok(Self {
            base: parts[0].trim().to_uppercase(),
            quote: parts[1].trim().to_uppercase(),
        })
    }
}

impl TryFrom<String> for Pair {
    type Error = anyhow::Error;

    fn try_from(pair_id: String) -> anyhow::Result<Self> {
        Self::try_from(pair_id.as_str())
    }
}

impl TryFrom<(String, String)> for Pair {
    type Error = anyhow::Error;

    fn try_from(pair: (String, String)) -> anyhow::Result<Self> {
        let (base, quote) = pair;

        if !base.chars().all(|c| c.is_ascii_alphabetic()) {
            anyhow::bail!("Invalid base symbol: only ASCII letters allowed");
        }

        if !quote.chars().all(|c| c.is_ascii_alphabetic()) {
            anyhow::bail!("Invalid quote symbol: only ASCII letters allowed");
        }

        Ok(Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        })
    }
}

impl FromStr for Pair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[macro_export]
macro_rules! pair {
    ($pair_str:expr) => {{
        // Compile-time validation
        #[allow(dead_code)]
        const fn is_valid_pair(s: &str) -> bool {
            let bytes = s.as_bytes();
            let mut count = 0;
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'/' || bytes[i] == b'-' || bytes[i] == b'_' {
                    count += 1;
                }
                i += 1;
            }
            count == 1
        }

        const _: () = {
            assert!(
                is_valid_pair($pair_str),
                "Invalid pair format. Expected format: BASE/QUOTE, BASE-QUOTE, or BASE_QUOTE"
            );
        };

        // Runtime normalization and parsing
        let normalized = $pair_str.replace(['-', '_'], "/");
        let mut parts = normalized.splitn(2, '/');
        let base = parts.next().unwrap().trim().to_uppercase();
        let quote = parts.next().unwrap().trim().to_uppercase();

        $crate::pair::Pair { base, quote }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Test `from_stable_pair` with various inputs
    #[rstest]
    #[case("BTCUSDT", Some(Pair { base: "BTC".to_string(), quote: "USD".to_string() }))]
    #[case("ETH-USDC", Some(Pair { base: "ETH".to_string(), quote: "USD".to_string() }))]
    #[case("SOL_USDT", Some(Pair { base: "SOL".to_string(), quote: "USD".to_string() }))]
    #[case("XRP/USD", Some(Pair { base: "XRP".to_string(), quote: "USD".to_string() }))]
    #[case("BTC/ETH", None)] // No stable suffix
    #[case("USDUSDT", Some(Pair { base: "USD".to_string(), quote: "USD".to_string() }))]
    #[case("USDTUSD", Some(Pair { base: "USDT".to_string(), quote: "USD".to_string() }))]
    #[case("btc_usdt", Some(Pair { base: "BTC".to_string(), quote: "USD".to_string() }))]
    #[case("EthDai", Some(Pair { base: "ETH".to_string(), quote: "USD".to_string() }))]
    #[case("", None)] // Empty string
    #[case("BTC", None)] // No stable suffix
    #[case("USDT", Some(Pair { base: "".to_string(), quote: "USD".to_string() }))]
    fn test_from_stable_pair(#[case] input: &str, #[case] expected: Option<Pair>) {
        assert_eq!(Pair::from_stable_pair(input), expected);
    }

    /// Test `create_routed_pair` with pairs sharing a common quote
    #[rstest]
    #[case(
        Pair { base: "BTC".to_string(), quote: "USD".to_string() },
        Pair { base: "ETH".to_string(), quote: "USD".to_string() },
        Pair { base: "BTC".to_string(), quote: "ETH".to_string() }
    )]
    #[case(
        Pair { base: "SOL".to_string(), quote: "USDT".to_string() },
        Pair { base: "LUNA".to_string(), quote: "USDT".to_string() },
        Pair { base: "SOL".to_string(), quote: "LUNA".to_string() }
    )]
    fn test_create_routed_pair(
        #[case] base_pair: Pair,
        #[case] quote_pair: Pair,
        #[case] expected: Pair,
    ) {
        assert_eq!(Pair::create_routed_pair(&base_pair, &quote_pair), expected);
    }

    /// Test `from_currencies` with different case inputs
    #[rstest]
    #[case("btc", "usd", Pair { base: "BTC".to_string(), quote: "USD".to_string() })]
    #[case("Eth", "Dai", Pair { base: "ETH".to_string(), quote: "DAI".to_string() })]
    #[case("sol", "usdt", Pair { base: "SOL".to_string(), quote: "USDT".to_string() })]
    fn test_from_currencies(#[case] base: &str, #[case] quote: &str, #[case] expected: Pair) {
        assert_eq!(Pair::from_currencies(base, quote), expected);
    }

    /// Test `as_tuple` returns the correct tuple
    #[rstest]
    #[case(Pair { base: "BTC".to_string(), quote: "USD".to_string() }, ("BTC".to_string(), "USD".to_string()))]
    #[case(Pair { base: "ETH".to_string(), quote: "USDT".to_string() }, ("ETH".to_string(), "USDT".to_string()))]
    fn test_as_tuple(#[case] pair: Pair, #[case] expected: (String, String)) {
        assert_eq!(pair.as_tuple(), expected);
    }

    /// Test `format_with_separator` with different separators
    #[rstest]
    #[case(Pair { base: "BTC".to_string(), quote: "USD".to_string() }, "/", "BTC/USD")]
    #[case(Pair { base: "ETH".to_string(), quote: "USDT".to_string() }, "-", "ETH-USDT")]
    #[case(Pair { base: "SOL".to_string(), quote: "USDC".to_string() }, "_", "SOL_USDC")]
    fn test_format_with_separator(
        #[case] pair: Pair,
        #[case] separator: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(pair.format_with_separator(separator), expected);
    }

    /// Test `to_pair_id` uses the standard "/" separator
    #[rstest]
    #[case(Pair { base: "BTC".to_string(), quote: "USD".to_string() }, "BTC/USD")]
    #[case(Pair { base: "ETH".to_string(), quote: "USDT".to_string() }, "ETH/USDT")]
    fn test_to_pair_id(#[case] pair: Pair, #[case] expected: &str) {
        assert_eq!(pair.to_pair_id(), expected);
    }

    /// Test `Display` implementation
    #[rstest]
    #[case(Pair { base: "BTC".to_string(), quote: "USD".to_string() }, "BTC/USD")]
    #[case(Pair { base: "ETH".to_string(), quote: "USDT".to_string() }, "ETH/USDT")]
    fn test_display(#[case] pair: Pair, #[case] expected: &str) {
        assert_eq!(format!("{pair}"), expected);
    }

    /// Test `From<Pair> for String`
    #[rstest]
    #[case(Pair { base: "BTC".to_string(), quote: "USD".to_string() }, "BTC/USD")]
    #[case(Pair { base: "ETH".to_string(), quote: "USDT".to_string() }, "ETH/USDT")]
    fn test_from_pair_to_string(#[case] pair: Pair, #[case] expected: &str) {
        let s: String = pair.into();
        assert_eq!(s, expected);
    }

    /// Test `From<&str> for Pair` with different separators and whitespace
    #[rstest]
    #[case("BTC/USD", Pair { base: "BTC".to_string(), quote: "USD".to_string() })]
    #[case("ETH-USDT", Pair { base: "ETH".to_string(), quote: "USDT".to_string() })]
    #[case("SOL_USDC", Pair { base: "SOL".to_string(), quote: "USDC".to_string() })]
    #[case(" btc / usd ", Pair { base: "BTC".to_string(), quote: "USD".to_string() })]
    fn test_from_str_to_pair(#[case] input: &str, #[case] expected: Pair) {
        let pair: Pair = input.try_into().unwrap();
        assert_eq!(pair, expected);
    }

    /// Test `From<String> for Pair`
    #[rstest]
    #[case("BTC/USD".to_string(), Pair { base: "BTC".to_string(), quote: "USD".to_string() })]
    #[case("ETH-USDT".to_string(), Pair { base: "ETH".to_string(), quote: "USDT".to_string() })]
    fn test_from_string_to_pair(#[case] input: String, #[case] expected: Pair) {
        let pair: Pair = input.try_into().unwrap();
        assert_eq!(pair, expected);
    }

    /// Test `FromStr for Pair`
    #[rstest]
    #[case("BTC/USD", Pair { base: "BTC".to_string(), quote: "USD".to_string() })]
    #[case("ETH-USDT", Pair { base: "ETH".to_string(), quote: "USDT".to_string() })]
    fn test_fromstr(#[case] input: &str, #[case] expected: Pair) {
        let pair: Pair = input.parse().unwrap();
        assert_eq!(pair, expected);
    }

    /// Test `From<(String, String)> for Pair`
    #[rstest]
    #[case(("btc".to_string(), "usd".to_string()), Pair { base: "BTC".to_string(), quote: "USD".to_string() })]
    #[case(("Eth".to_string(), "Dai".to_string()), Pair { base: "ETH".to_string(), quote: "DAI".to_string() })]
    fn test_from_tuple(#[case] input: (String, String), #[case] expected: Pair) {
        let pair: Pair = input.try_into().unwrap();
        assert_eq!(pair, expected);
    }

    /// Test the `pair!` macro with valid inputs
    #[test]
    fn test_pair_macro() {
        assert_eq!(
            pair!("BTC/USD"),
            Pair {
                base: "BTC".to_string(),
                quote: "USD".to_string()
            }
        );
        assert_eq!(
            pair!("ETH-USDT"),
            Pair {
                base: "ETH".to_string(),
                quote: "USDT".to_string()
            }
        );
        assert_eq!(
            pair!("SOL_USDC"),
            Pair {
                base: "SOL".to_string(),
                quote: "USDC".to_string()
            }
        );
        assert_eq!(
            pair!(" btc / usd "),
            Pair {
                base: "BTC".to_string(),
                quote: "USD".to_string()
            }
        );
    }

    /// Test the `Default` implementation
    #[test]
    fn test_default() {
        assert_eq!(
            Pair::default(),
            Pair {
                base: "".to_string(),
                quote: "".to_string()
            }
        );
    }
}
