#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub enum AggregationMode {
    #[cfg_attr(feature = "serde", serde(rename = "median"))]
    Median,
    #[cfg_attr(feature = "serde", serde(rename = "twap"))]
    #[default]
    Twap,
}

impl AggregationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Median => "median",
            Self::Twap => "twap",
        }
    }
}
