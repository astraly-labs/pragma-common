#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum StarknetNetwork {
    #[cfg_attr(feature = "serde", serde(rename = "starknet-mainnet"))]
    Mainnet,
    #[cfg_attr(feature = "serde", serde(rename = "starknet-sepolia"))]
    Sepolia,
}

impl std::fmt::Display for StarknetNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "starknet-mainnet"),
            Self::Sepolia => write!(f, "starknet-sepolia"),
        }
    }
}

impl Default for StarknetNetwork {
    fn default() -> Self {
        Self::Mainnet
    }
}
