use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FuturesContractParseError {
    #[error("futures contract symbol is empty")]
    Empty,
    #[error("futures contract symbol is missing a month code")]
    MissingMonthCode,
    #[error("futures contract root is invalid: {0}")]
    InvalidRoot(String),
    #[error("invalid futures month code: {0}")]
    InvalidMonthCode(char),
    #[error("futures contract symbol is missing a year suffix")]
    MissingYear,
    #[error("invalid futures year suffix: {0}")]
    InvalidYear(String),
    #[error(
        "futures contract {contract} is too far in the past for reference date {reference_date}"
    )]
    TooFarInPast {
        contract: String,
        reference_date: String,
    },
    #[error(
        "futures contract {contract} is too far in the future for reference date {reference_date}"
    )]
    TooFarInFuture {
        contract: String,
        reference_date: String,
    },
}
