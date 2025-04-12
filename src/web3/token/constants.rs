use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::web3::Chain;

use super::Token;

static ETH_LOCK: OnceLock<Token> = OnceLock::new();
static SOL_LOCK: OnceLock<Token> = OnceLock::new();
static SUI_LOCK: OnceLock<Token> = OnceLock::new();
static APT_LOCK: OnceLock<Token> = OnceLock::new();
static POL_LOCK: OnceLock<Token> = OnceLock::new();
static BNB_LOCK: OnceLock<Token> = OnceLock::new();
static AVAX_LOCK: OnceLock<Token> = OnceLock::new();
static XDAI_LOCK: OnceLock<Token> = OnceLock::new();
static WLD_LOCK: OnceLock<Token> = OnceLock::new();
static USDT_LOCK: OnceLock<Token> = OnceLock::new();
static USDC_LOCK: OnceLock<Token> = OnceLock::new();
static AAVE_LOCK: OnceLock<Token> = OnceLock::new();
static BTC_LOCK: OnceLock<Token> = OnceLock::new();
static JLP_LOCK: OnceLock<Token> = OnceLock::new();

#[allow(non_snake_case)]
#[must_use]
pub fn ETH() -> Token {
    ETH_LOCK
        .get_or_init(|| Token {
            name: "Ethereum".to_string(),
            ticker: "ETH".to_string(),
            decimals: 18,
            addresses: Some(BTreeMap::from([
                (
                    Chain::Starknet,
                    "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"
                        .to_string(),
                ),
                (
                    Chain::Ethereum,
                    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string(),
                ),
                (
                    Chain::Base,
                    "0x4200000000000000000000000000000000000006".to_string(),
                ),
                (
                    Chain::Optimism,
                    "0x4200000000000000000000000000000000000006".to_string(),
                ),
                (
                    Chain::Arbitrum,
                    "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1".to_string(),
                ),
                (
                    Chain::ZkSync,
                    "0x000000000000000000000000000000000000800A".to_string(),
                ),
            ])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn SOL() -> Token {
    SOL_LOCK
        .get_or_init(|| Token {
            name: "Solana".to_string(),
            ticker: "SOL".to_string(),
            decimals: 9,
            addresses: Some(BTreeMap::from([(
                Chain::Solana,
                "So11111111111111111111111111111111111111112".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn SUI() -> Token {
    SUI_LOCK
        .get_or_init(|| Token {
            name: "Sui".to_string(),
            ticker: "SUI".to_string(),
            decimals: 9,
            addresses: Some(BTreeMap::from([(Chain::Sui, "0x2::sui::SUI".to_string())])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn APT() -> Token {
    APT_LOCK
        .get_or_init(|| Token {
            name: "Aptos".to_string(),
            ticker: "APT".to_string(),
            decimals: 8,
            addresses: Some(BTreeMap::from([(
                Chain::Aptos,
                "0x1::aptos_coin::AptosCoin".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn POL() -> Token {
    POL_LOCK
        .get_or_init(|| Token {
            name: "Polygon".to_string(),
            ticker: "POL".to_string(),
            decimals: 18,
            addresses: Some(BTreeMap::from([(
                Chain::Polygon,
                "0x0000000000000000000000000000000000001010".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn BNB() -> Token {
    BNB_LOCK
        .get_or_init(|| Token {
            name: "BNB".to_string(),
            ticker: "BNB".to_string(),
            decimals: 18,
            addresses: Some(BTreeMap::from([(
                Chain::Bnb,
                "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn AVAX() -> Token {
    AVAX_LOCK
        .get_or_init(|| Token {
            name: "Avalanche".to_string(),
            ticker: "AVAX".to_string(),
            decimals: 18,
            addresses: Some(BTreeMap::from([(
                Chain::Avalanche,
                "0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn XDAI() -> Token {
    XDAI_LOCK
        .get_or_init(|| Token {
            name: "xDAI".to_string(),
            ticker: "xDAI".to_string(),
            decimals: 18,
            addresses: Some(BTreeMap::from([(
                Chain::Gnosis,
                "0xe91d153e0b41518a2ce8dd3d7944fa863463a97d".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn WLD() -> Token {
    WLD_LOCK
        .get_or_init(|| Token {
            name: "Worldcoin".to_string(),
            ticker: "WLD".to_string(),
            decimals: 18,
            addresses: Some(BTreeMap::from([(
                Chain::Worldchain,
                "0x2cfc85d8e48f8eab294be644d9e25c3030863003".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn USDT() -> Token {
    USDT_LOCK.get_or_init(|| Token {
        name: "Tether USD".to_string(),
        ticker: "USDT".to_string(),
        decimals: 6,
        addresses: Some(BTreeMap::from([
            (
                Chain::Ethereum,
                "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            ),
            (
                Chain::Polygon,
                "0xc2132D05D31c914a87C6611C10748AEb04B58e8F".to_string(),
            ),
            (
                Chain::Optimism,
                "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58".to_string(),
            ),
            (
                Chain::Arbitrum,
                "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9".to_string(),
            ),
            (
                Chain::Avalanche,
                "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7".to_string(),
            ),
            (
                Chain::Gnosis,
                "0x4ECaBa5870353805aimetic068101A40E0f32ed605C6".to_string(),
            ),
            (
                Chain::Aptos,
                "0x9770fa9c725cbd97eb50b2be5f7416efdfd1f1554beb0750d4dae4c64e860da3::fa_to_coin_wrapper::WrappedUSDT".to_string(),
            ),
        ])),
    }).clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn USDC() -> Token {
    USDC_LOCK.get_or_init(|| Token {
        name: "Circle USD".to_string(),
        ticker: "USDC".to_string(),
        decimals: 6,
        addresses: Some(BTreeMap::from([
            (
                Chain::Base,
                "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
            ),
            (
                Chain::ZkSync,
                "0x1d17CBcF0D6D143135aE902365D2E5e2A16538D4".to_string(),
            ),
            (
                Chain::Solana,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            ),
            (
                Chain::Starknet,
                "0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8".to_string(),
            ),
            (
                Chain::Sui,
                "0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN".to_string(),
            ),
            (
                Chain::Worldchain,
                "0x79A02482A880bCE3F13e09Da970dC34db4CD24d1".to_string(),
            ),
            (
                Chain::Bnb,
                "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d".to_string(),
            ),
            (
                Chain::Ethereum,
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            ),
        ])),
    }).clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn AAVE() -> Token {
    AAVE_LOCK
        .get_or_init(|| Token {
            name: "Aave".to_string(),
            ticker: "AAVE".to_string(),
            decimals: 18,
            addresses: Some(BTreeMap::from([(
                Chain::Ethereum,
                "0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn BTC() -> Token {
    BTC_LOCK
        .get_or_init(|| Token {
            name: "Bitcoin".to_string(),
            ticker: "BTC".to_string(),
            decimals: 8,
            addresses: Some(BTreeMap::from([(
                Chain::Ethereum,
                "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string(),
            )])),
        })
        .clone()
}

#[allow(non_snake_case)]
#[must_use]
pub fn JLP() -> Token {
    JLP_LOCK
        .get_or_init(|| Token {
            name: "JLP".to_string(),
            ticker: "JLP".to_string(),
            decimals: 18,
            addresses: Some(BTreeMap::from([(
                Chain::Solana,
                "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4".to_string(),
            )])),
        })
        .clone()
}
