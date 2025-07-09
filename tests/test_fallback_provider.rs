#[cfg(feature = "starknet")]
#[tokio::test]
async fn test_fallback() {
    use std::str::FromStr;

    use pragma_common::starknet::fallback_provider::FallbackProvider;
    use starknet::{
        macros::felt_hex,
        providers::{Provider, Url},
    };

    let provider = FallbackProvider::new(vec![
        Url::from_str("http://i_do_not_exists_hehe.com").unwrap(),
        Url::from_str("https://api.cartridge.gg/x/starknet/mainnet/").unwrap(),
    ])
    .unwrap();

    let chain_id = provider.chain_id().await.unwrap();
    assert_eq!(chain_id, felt_hex!("0x534e5f4d41494e"))
}
