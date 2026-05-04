mod builder;
mod errors;
#[cfg(feature = "proto")]
mod proto;
mod types;

pub use builder::FuturesContractBuilder;
pub use errors::FuturesContractParseError;
pub use types::{Contract, FuturesContract, FuturesMonth, FuturesRoot, YearFormat};

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("plj6", "PL", FuturesMonth::April, 2026, "PLJ6")]
    #[case("NGK26", "NG", FuturesMonth::May, 2026, "NGK26")]
    #[case("RTYM6", "RTY", FuturesMonth::June, 2026, "RTYM6")]
    #[case("6EZ6", "6E", FuturesMonth::December, 2026, "6EZ6")]
    #[case("ABCJ6", "ABC", FuturesMonth::April, 2026, "ABCJ6")]
    fn parses_supported_raw_contracts(
        #[case] raw: &str,
        #[case] root: &str,
        #[case] month: FuturesMonth,
        #[case] year: u16,
        #[case] display: &str,
    ) {
        let contract = FuturesContractBuilder::raw(raw).build().unwrap();

        assert_eq!(contract.root, FuturesRoot::new(root).unwrap());
        assert_eq!(contract.month, month);
        assert_eq!(contract.year, year);
        assert_eq!(contract.to_string(), display);
    }

    #[rstest]
    #[case("GC", FuturesContractParseError::MissingYear)]
    #[case("6", FuturesContractParseError::MissingMonthCode)]
    #[case("GCA6", FuturesContractParseError::InvalidMonthCode('A'))]
    #[case("P.J6", FuturesContractParseError::InvalidRoot("P.".to_string()))]
    #[case("PLA6", FuturesContractParseError::InvalidMonthCode('A'))]
    #[case("PLJ260", FuturesContractParseError::InvalidYear("260".to_string()))]
    fn rejects_invalid_contracts(#[case] raw: &str, #[case] expected: FuturesContractParseError) {
        let error = FuturesContractBuilder::raw(raw).build().unwrap_err();

        match (error, expected) {
            (
                FuturesContractParseError::InvalidYear(actual),
                FuturesContractParseError::InvalidYear(expected),
            ) => assert_eq!(actual, expected),
            (
                FuturesContractParseError::InvalidRoot(actual),
                FuturesContractParseError::InvalidRoot(expected),
            ) => assert_eq!(actual, expected),
            (actual, expected) => assert_eq!(actual, expected),
        }
    }

    #[test]
    fn parses_activ_contracts() {
        let contract = FuturesContractBuilder::activ("GC/26M").unwrap();

        assert_eq!(contract.root, FuturesRoot::new("GC").unwrap());
        assert_eq!(contract.month, FuturesMonth::June);
        assert_eq!(contract.year, 2026);
        assert_eq!(contract.raw_symbol(), "GCM26");
    }

    #[test]
    fn contract_enum_exposes_raw_symbol() {
        let contract = Contract::from_raw_symbol("CLK6").unwrap();

        assert_eq!(contract.raw_symbol(), "CLK6");
        assert_eq!(contract.to_string(), "CLK6");
    }

    #[test]
    fn contract_validation_accepts_reasonable_reference_date() {
        let contract = FuturesContractBuilder::raw("PLJ6").build().unwrap();
        let reference_date = NaiveDate::from_ymd_opt(2026, 3, 24).unwrap();

        assert_eq!(contract.validate_against_date(reference_date), Ok(()));
    }

    #[test]
    fn contract_validation_rejects_contracts_too_far_in_past() {
        let contract = FuturesContractBuilder::raw("PLG26").build().unwrap();
        let reference_date = NaiveDate::from_ymd_opt(2026, 3, 24).unwrap();

        assert_eq!(
            contract.validate_against_date(reference_date),
            Err(FuturesContractParseError::TooFarInPast {
                contract: "PLG26".to_string(),
                reference_date: reference_date.to_string(),
            })
        );
    }

    #[test]
    fn contract_validation_rejects_contracts_too_far_in_future() {
        let contract = FuturesContractBuilder::raw("PLJ27").build().unwrap();
        let reference_date = NaiveDate::from_ymd_opt(2026, 3, 24).unwrap();

        assert_eq!(
            contract.validate_against_date(reference_date),
            Err(FuturesContractParseError::TooFarInFuture {
                contract: "PLJ27".to_string(),
                reference_date: reference_date.to_string(),
            })
        );
    }
}
