use std::{fmt::Display, str::FromStr};

#[derive(Debug, thiserror::Error)]
pub enum InstrumentTypeError {
    #[error("Unknown instrument_type")]
    Unknown,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum InstrumentType {
    Spot,
    Perp,
}

impl InstrumentType {
    pub const ALL: [Self; 2] = [Self::Spot, Self::Perp];

    pub const fn to_id(&self) -> i32 {
        match self {
            Self::Spot => 1,
            Self::Perp => 2,
        }
    }

    pub const fn is_spot(&self) -> bool {
        match self {
            Self::Spot => true,
            Self::Perp => false,
        }
    }

    pub const fn is_perp(&self) -> bool {
        match self {
            Self::Spot => false,
            Self::Perp => true,
        }
    }
}

impl Display for InstrumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spot => write!(f, "spot"),
            Self::Perp => write!(f, "perp"),
        }
    }
}

impl TryFrom<i32> for InstrumentType {
    type Error = InstrumentTypeError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Spot),
            2 => Ok(Self::Perp),
            _ => Err(InstrumentTypeError::Unknown),
        }
    }
}

impl FromStr for InstrumentType {
    type Err = InstrumentTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spot" => Ok(Self::Spot),
            "perp" => Ok(Self::Perp),
            _ => Err(InstrumentTypeError::Unknown),
        }
    }
}
