use std::str::FromStr;

#[derive(Debug, Copy, Hash, Eq, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum Chain {
    Starknet,
    Solana,
    Sui,
    Aptos,
    // Evm chains
    Ethereum,
    Base,
    Arbitrum,
    Optimism,
    ZkSync,
    Polygon,
    Bnb,
    Avalanche,
    Gnosis,
    Worldchain,
}

impl Chain {
    #[must_use]
    pub const fn id(&self) -> Option<u64> {
        let chain_id = match self {
            Self::Ethereum => 1,
            Self::Optimism => 10,
            Self::Polygon => 137,
            Self::ZkSync => 324,
            Self::Base => 8453,
            Self::Arbitrum => 42161,
            Self::Bnb => 56,
            Self::Avalanche => 43114,
            Self::Gnosis => 100,
            Self::Worldchain => 480,
            _ => {
                return None;
            }
        };

        Some(chain_id)
    }

    pub const fn is_evm(&self) -> bool {
        matches!(
            self,
            Self::Ethereum
                | Self::Optimism
                | Self::Polygon
                | Self::ZkSync
                | Self::Base
                | Self::Arbitrum
                | Self::Bnb
                | Self::Avalanche
                | Self::Gnosis
                | Self::Worldchain
        )
    }

    pub fn from_id(id: u64) -> Result<Self, ChainError> {
        match id {
            1 => Ok(Self::Ethereum),
            10 => Ok(Self::Optimism),
            137 => Ok(Self::Polygon),
            324 => Ok(Self::ZkSync),
            8453 => Ok(Self::Base),
            42161 => Ok(Self::Arbitrum),
            56 => Ok(Self::Bnb),
            43114 => Ok(Self::Avalanche),
            100 => Ok(Self::Gnosis),
            480 => Ok(Self::Worldchain),
            _ => Err(ChainError::UnknownChainId(id.to_string())),
        }
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Chain {
    type Err = ChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "starknet" => Ok(Self::Starknet),
            "solana" => Ok(Self::Solana),
            "sui" => Ok(Self::Sui),
            "aptos" => Ok(Self::Aptos),
            "ethereum" => Ok(Self::Ethereum),
            "base" => Ok(Self::Base),
            "arbitrum" => Ok(Self::Arbitrum),
            "optimism" => Ok(Self::Optimism),
            "zksync" => Ok(Self::ZkSync),
            "polygon" => Ok(Self::Polygon),
            "bnb" => Ok(Self::Bnb),
            "avalanche" => Ok(Self::Avalanche),
            "gnosis" => Ok(Self::Gnosis),
            "worldchain" => Ok(Self::Worldchain),
            _ => Err(ChainError::UnknownChainId(s.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("Unknown chain id: {0}")]
    UnknownChainId(String),
}
