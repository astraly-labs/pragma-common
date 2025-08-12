#[derive(Debug, thiserror::Error)]
pub enum InstrumentTypeError {
    #[error("Unknown instrument_type")]
    Unknown,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy, strum::EnumString, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum InstrumentType {
    #[default]
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

    pub const fn from_str_const(s: &str) -> Option<Self> {
        match s.as_bytes() {
            b"spot" | b"SPOT" | b"Spot" => Some(Self::Spot),
            b"perp" | b"PERP" | b"Perp" => Some(Self::Perp),
            _ => None,
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
