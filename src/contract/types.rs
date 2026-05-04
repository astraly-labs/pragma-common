use std::fmt;

use chrono::Datelike;

use super::{FuturesContractBuilder, FuturesContractParseError};

const MAX_MONTHS_IN_PAST: i32 = 0;
const MAX_MONTHS_IN_FUTURE: i32 = 12;
const MAX_FUTURES_ROOT_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum Contract {
    Futures(FuturesContract),
}

impl Contract {
    pub fn from_raw_symbol(symbol: &str) -> Result<Self, FuturesContractParseError> {
        FuturesContractBuilder::raw(symbol)
            .build()
            .map(Contract::Futures)
    }

    pub fn from_cme_symbol(symbol: &str) -> Result<Self, FuturesContractParseError> {
        Self::from_raw_symbol(symbol)
    }

    pub fn from_activ_symbol(symbol: &str) -> Result<Self, FuturesContractParseError> {
        FuturesContractBuilder::activ(symbol).map(Contract::Futures)
    }

    pub fn raw_symbol(&self) -> String {
        match self {
            Self::Futures(contract) => contract.raw_symbol(),
        }
    }

    pub fn contract_month_yyyymm(&self) -> String {
        match self {
            Self::Futures(contract) => contract.contract_month_yyyymm(),
        }
    }

    pub fn validate_against_date(
        &self,
        date: chrono::NaiveDate,
    ) -> Result<(), FuturesContractParseError> {
        match self {
            Self::Futures(contract) => contract.validate_against_date(date),
        }
    }
}

impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_symbol())
    }
}

impl std::str::FromStr for Contract {
    type Err = FuturesContractParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_raw_symbol(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FuturesContract {
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    pub root: FuturesRoot,
    pub month: FuturesMonth,
    pub year: u16,
}

impl FuturesContract {
    pub fn raw_symbol(&self) -> String {
        format!(
            "{}{}{:02}",
            self.root.cme_code(),
            self.month.code(),
            self.year % 100
        )
    }

    pub fn contract_month_yyyymm(&self) -> String {
        format!("{}{:02}", self.year, self.month.number())
    }

    pub fn validate_against_date(
        &self,
        date: chrono::NaiveDate,
    ) -> Result<(), FuturesContractParseError> {
        let contract_month_index = contract_month_index(self.year, self.month);
        let reference_month_index = date_month_index(date);
        let months_from_reference = contract_month_index - reference_month_index;

        if months_from_reference < -MAX_MONTHS_IN_PAST {
            return Err(FuturesContractParseError::TooFarInPast {
                contract: self.raw_symbol(),
                reference_date: date.to_string(),
            });
        }

        if months_from_reference > MAX_MONTHS_IN_FUTURE {
            return Err(FuturesContractParseError::TooFarInFuture {
                contract: self.raw_symbol(),
                reference_date: date.to_string(),
            });
        }

        Ok(())
    }
}

impl fmt::Display for FuturesContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_symbol())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FuturesRoot {
    len: u8,
    bytes: [u8; MAX_FUTURES_ROOT_LEN],
}

impl FuturesRoot {
    pub fn new(root: &str) -> Result<Self, FuturesContractParseError> {
        let normalized = root.trim().to_ascii_uppercase();
        if normalized.is_empty()
            || normalized.len() > MAX_FUTURES_ROOT_LEN
            || !normalized.bytes().all(|byte| byte.is_ascii_alphanumeric())
        {
            return Err(FuturesContractParseError::InvalidRoot(root.to_string()));
        }

        let mut bytes = [0; MAX_FUTURES_ROOT_LEN];
        bytes[..normalized.len()].copy_from_slice(normalized.as_bytes());

        Ok(Self {
            len: u8::try_from(normalized.len()).expect("MAX_FUTURES_ROOT_LEN must fit in u8"),
            bytes,
        })
    }

    pub fn cme_code(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)])
            .expect("FuturesRoot is validated as ASCII")
    }
}

impl fmt::Display for FuturesRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cme_code())
    }
}

impl std::str::FromStr for FuturesRoot {
    type Err = FuturesContractParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for FuturesRoot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.cme_code())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FuturesRoot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let root = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::new(&root).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum FuturesMonth {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl FuturesMonth {
    pub fn code(self) -> char {
        match self {
            Self::January => 'F',
            Self::February => 'G',
            Self::March => 'H',
            Self::April => 'J',
            Self::May => 'K',
            Self::June => 'M',
            Self::July => 'N',
            Self::August => 'Q',
            Self::September => 'U',
            Self::October => 'V',
            Self::November => 'X',
            Self::December => 'Z',
        }
    }

    pub fn from_code(code: char) -> Option<Self> {
        match code.to_ascii_uppercase() {
            'F' => Some(Self::January),
            'G' => Some(Self::February),
            'H' => Some(Self::March),
            'J' => Some(Self::April),
            'K' => Some(Self::May),
            'M' => Some(Self::June),
            'N' => Some(Self::July),
            'Q' => Some(Self::August),
            'U' => Some(Self::September),
            'V' => Some(Self::October),
            'X' => Some(Self::November),
            'Z' => Some(Self::December),
            _ => None,
        }
    }

    pub const fn number(self) -> u8 {
        match self {
            Self::January => 1,
            Self::February => 2,
            Self::March => 3,
            Self::April => 4,
            Self::May => 5,
            Self::June => 6,
            Self::July => 7,
            Self::August => 8,
            Self::September => 9,
            Self::October => 10,
            Self::November => 11,
            Self::December => 12,
        }
    }
}

impl fmt::Display for FuturesMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

fn contract_month_index(year: u16, month: FuturesMonth) -> i32 {
    i32::from(year) * 12 + i32::from(month.number())
}

fn date_month_index(date: chrono::NaiveDate) -> i32 {
    date.year() * 12 + i32::try_from(date.month()).expect("chrono month must fit in i32")
}
