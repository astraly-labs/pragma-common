#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub enum Network {
    #[cfg_attr(feature = "serde", serde(rename = "sepolia"))]
    Sepolia,
    #[cfg_attr(feature = "serde", serde(rename = "mainnet"))]
    Mainnet,
}

impl Network {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sepolia => "sepolia",
            Self::Mainnet => "mainnet",
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::Mainnet
    }
}
