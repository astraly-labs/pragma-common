#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, strum::Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Currency {
    USD,
    BTC,
    ETH,
    wstETH,
    USDC,
    STRK,
    #[strum(to_string = "rUSDC-stark")]
    rUSDC,
}