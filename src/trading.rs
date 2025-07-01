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
pub enum Side {
    Long,
    Short,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Long => Side::Short,
            Side::Short => Side::Long,
        }
    }
}
