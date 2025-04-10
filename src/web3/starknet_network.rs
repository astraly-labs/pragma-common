#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub enum StarknetNetwork {
    #[cfg_attr(feature = "serde", serde(rename = "sepolia"))]
    Sepolia,
    #[cfg_attr(feature = "serde", serde(rename = "mainnet"))]
    Mainnet,
}

impl StarknetNetwork {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sepolia => "sepolia",
            Self::Mainnet => "mainnet",
        }
    }
}

impl Default for StarknetNetwork {
    fn default() -> Self {
        Self::Mainnet
    }
}
