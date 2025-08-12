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
pub enum Currency {
    USD,
    EUR,
    ETH,
    WSTETH,
    USDC,
    USDT,
}

impl Currency {
    pub fn to_ascii_uppercase(&self) -> String {
        format!("{}", self).to_ascii_uppercase()
    }

    pub const fn from_str_const(s: &str) -> Option<Self> {
        match s.as_bytes() {
            b"USD" => Some(Currency::USD),
            b"EUR" => Some(Currency::EUR),
            _ => None,
        }
    }
}

/// Macro for compile-time validation of currency strings.
///
/// ```compile_fail
/// use md_types::currency::Currency;
/// let currency = currency!("UNKNOWN");
/// ```
#[allow(unused_macros)]
macro_rules! currency {
    ($s:literal) => {{
        const _: () = {
            if Currency::from_str_const($s).is_none() {
                panic!(concat!("Invalid Currency: ", $s));
            }
        };
        // Use the strum-generated FromStr at runtime
        <Currency as std::str::FromStr>::from_str($s).unwrap()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_currency_creation() {
        let currency = currency!("USD");
        assert_eq!(currency, Currency::USD);

        let currency = currency!("EUR");
        assert_eq!(currency, Currency::EUR);
    }
}
