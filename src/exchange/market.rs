use std::fmt::Display;

use derive_more::Constructor;

use crate::{exchange::currency::Currency, Exchange, InstrumentType};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Constructor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Market {
    /// Raw name of the market, e.g., "BTC" or "BTC-USD-PERP"
    raw_name: String,
    /// Base asset symbol, e.g., "BTC"
    base: Currency,
    /// Quote asset symbol, e.g., "USD"
    quote: Currency,
    /// Instrument type, e.g., "PERP"
    mtype: InstrumentType,
    /// Venue, e.g., "Extended"
    exchange: Exchange,
}

impl Display for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.base(), self.quote(), self.mtype())
    }
}

impl Market {
    pub fn raw_name(&self) -> String {
        self.raw_name.clone()
    }

    pub fn base(&self) -> &Currency {
        &self.base
    }

    pub fn quote(&self) -> &Currency {
        &self.quote
    }

    pub fn mtype(&self) -> &InstrumentType {
        &self.mtype
    }

    pub fn exchange(&self) -> &Exchange {
        &self.exchange
    }
}

/// Create a Market instance with compile-time validation.
///
/// This macro ensures that only valid currencies, instrument types, and venues
/// can be used, failing at compile time for invalid values.
///
/// # Examples
///
/// ```
/// use md_types::{market, market::Market};
///
/// // Valid market creation
/// let market = market!("USD", "EUR", "SPOT", "EXTENDED");
/// assert_eq!(market.raw_name(), "USD:EUR:SPOT");
///
/// let perp_market = market!("EUR", "USD", "Perp", "Lmax");
/// assert_eq!(perp_market.raw_name(), "EUR:USD:PERP");
/// ```
///
/// The following will fail at compile time:
///
/// ```compile_fail
/// use md_types::market;
///
/// // This will fail to compile - UNKNOWN is not a valid instrument type
/// let market = market!("USD", "EUR", "UNKNOWN", "Extended");
/// ```
///
/// ```compile_fail
/// use md_types::market;
///
/// // This will fail to compile - INVALID is not a valid currency
/// let market = market!("INVALID", "USD", "Spot", "Extended");
/// ```
///
/// ```compile_fail
/// use md_types::market;
///
/// // This will fail to compile - InvalidVenue is not a valid venue
/// let market = market!("USD", "EUR", "Spot", "InvalidVenue");
/// ```
#[macro_export]
macro_rules! market {
    ($base:literal, $quote:literal, $instrument:literal, $venue:literal) => {{
        use std::str::FromStr as _;

        // Compile-time validation
        const _: () = {
            if $crate::exchange::currency::Currency::from_str_const($base).is_none() {
                panic!(concat!("Invalid base currency: ", $base));
            }
            if $crate::exchange::currency::Currency::from_str_const($quote).is_none() {
                panic!(concat!("Invalid quote currency: ", $quote));
            }
            if $crate::instrument_type::InstrumentType::from_str_const($instrument).is_none() {
                panic!(concat!("Invalid instrument type: ", $instrument));
            }
            if $crate::exchange::Exchange::from_str_const($venue).is_none() {
                panic!(concat!("Invalid venue: ", $venue));
            }
        };

        // Runtime parsing (will succeed due to compile-time validation)
        let base = $crate::exchange::currency::Currency::from_str($base).unwrap();
        let quote = $crate::exchange::currency::Currency::from_str($quote).unwrap();
        let instrument_type =
            $crate::instrument_type::InstrumentType::from_str_const($instrument).unwrap();
        let venue = $crate::exchange::Exchange::from_str($venue).unwrap();

        let raw_name = format!(
            "{}:{}:{}",
            $base.to_uppercase(),
            $quote.to_uppercase(),
            $instrument.to_uppercase()
        );

        $crate::exchange::market::Market::new(raw_name, base, quote, instrument_type, venue)
    }};
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_valid_market_creation() {
        // Test with valid spot market
        let spot_market = market!("USD", "EUR", "SPOT", "EXTENDED");
        assert_eq!(spot_market.raw_name(), "USD:EUR:SPOT");
        assert_eq!(format!("{}", spot_market.base()), "USD");
        assert_eq!(format!("{}", spot_market.quote()), "EUR");
        assert_eq!(format!("{}", spot_market.mtype()), "spot");
        assert_eq!(format!("{}", spot_market.exchange()), "EXTENDED");

        // Test with valid perp market
        let perp_market = market!("EUR", "USD", "Perp", "Lmax");
        assert_eq!(perp_market.raw_name(), "EUR:USD:PERP");
        assert_eq!(format!("{}", perp_market.base()), "EUR");
        assert_eq!(format!("{}", perp_market.quote()), "USD");
        assert_eq!(format!("{}", perp_market.mtype()), "perp");
        assert_eq!(format!("{}", perp_market.exchange()), "LMAX");

        // Test with lowercase instrument type and venue
        let lowercase_market = market!("EUR", "USD", "spot", "extended");
        assert_eq!(lowercase_market.raw_name(), "EUR:USD:SPOT");
        assert_eq!(format!("{}", lowercase_market.mtype()), "spot");
    }
}
