use std::str::FromStr;

use super::{Token, APT, AVAX, BNB, ETH, POL, SOL, SUI, USDC, USDT, WLD, XDAI};

#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("Unknown chain: {0}")]
    UnknownChain(String),
}

#[derive(Debug, Copy, Hash, Eq, Clone, PartialEq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
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
    pub fn from_chain_id(id: u64) -> Option<Self> {
        match id {
            1 => Some(Self::Ethereum),
            10 => Some(Self::Optimism),
            137 => Some(Self::Polygon),
            324 => Some(Self::ZkSync),
            8453 => Some(Self::Base),
            42161 => Some(Self::Arbitrum),
            56 => Some(Self::Bnb),
            43114 => Some(Self::Avalanche),
            100 => Some(Self::Gnosis),
            480 => Some(Self::Worldchain),
            _ => None,
        }
    }

    #[must_use]
    pub const fn chain_id(&self) -> Option<u64> {
        match self {
            Self::Ethereum => Some(1),
            Self::Optimism => Some(10),
            Self::Polygon => Some(137),
            Self::ZkSync => Some(324),
            Self::Base => Some(8453),
            Self::Arbitrum => Some(42161),
            Self::Bnb => Some(56),
            Self::Avalanche => Some(43114),
            Self::Gnosis => Some(100),
            Self::Worldchain => Some(480),
            _ => None,
        }
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

    #[must_use]
    /// Returns the gas token for the chain
    pub fn gas_token(&self) -> Token {
        match self {
            Self::Ethereum
            | Self::Base
            | Self::Optimism
            | Self::Starknet
            | Self::Arbitrum
            | Self::ZkSync => ETH(),
            Self::Solana => SOL(),
            Self::Sui => SUI(),
            Self::Aptos => APT(),
            Self::Polygon => POL(),
            Self::Bnb => BNB(),
            Self::Avalanche => AVAX(),
            Self::Gnosis => XDAI(),
            Self::Worldchain => WLD(),
        }
    }

    /// Returns the main stablecoin for the chain (or None if there is none)
    pub fn usd_token(&self) -> Token {
        match self {
            Self::Polygon | Self::Optimism | Self::Arbitrum | Self::Gnosis | Self::Aptos => USDT(),
            Self::Ethereum
            | Self::Base
            | Self::ZkSync
            | Self::Bnb
            | Self::Avalanche
            | Self::Worldchain
            | Self::Solana
            | Self::Starknet
            | Self::Sui => USDC(),
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
            _ => Err(ChainError::UnknownChain(s.to_string())),
        }
    }
}
