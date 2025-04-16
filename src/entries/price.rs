#[cfg(feature = "capnp")]
use capnp::serialize;

#[cfg(feature = "capnp")]
use crate::schema_capnp;
use crate::{instrument_type::InstrumentType, pair::Pair, web3::Chain};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PriceEntry {
    pub source: String,
    pub chain: Option<Chain>,
    pub pair: Pair,
    pub timestamp_ms: i64,
    pub price: u128,
    pub volume: u128,
    pub expiration_timestamp: Option<i64>,
}

impl PriceEntry {
    pub fn instrument_type(&self) -> InstrumentType {
        match self.expiration_timestamp {
            None => InstrumentType::Spot,
            Some(_) => InstrumentType::Perp,
        }
    }
}

#[cfg(feature = "capnp")]
impl crate::CapnpSerialize for PriceEntry {
    fn to_capnp(&self) -> Vec<u8> {
        let mut message = capnp::message::Builder::new_default();
        let mut builder = message.init_root::<schema_capnp::price_entry::Builder>();

        // Set source
        builder.set_source(&self.source);

        // Set chain union
        let mut chain = builder.reborrow().init_chain();
        match &self.chain {
            Some(chain_value) => {
                chain.set_chain(match chain_value {
                    Chain::Starknet => schema_capnp::Chain::Starknet,
                    Chain::Solana => schema_capnp::Chain::Solana,
                    Chain::Sui => schema_capnp::Chain::Sui,
                    Chain::Aptos => schema_capnp::Chain::Aptos,
                    Chain::Ethereum => schema_capnp::Chain::Ethereum,
                    Chain::Base => schema_capnp::Chain::Base,
                    Chain::Arbitrum => schema_capnp::Chain::Arbitrum,
                    Chain::Optimism => schema_capnp::Chain::Optimism,
                    Chain::ZkSync => schema_capnp::Chain::Zksync,
                    Chain::Polygon => schema_capnp::Chain::Polygon,
                    Chain::Bnb => schema_capnp::Chain::Bnb,
                    Chain::Avalanche => schema_capnp::Chain::Avalanche,
                    Chain::Gnosis => schema_capnp::Chain::Gnosis,
                    Chain::Worldchain => schema_capnp::Chain::Worldchain,
                });
            }
            None => {
                chain.set_no_chain(());
            }
        };

        // Set pair
        let mut pair = builder.reborrow().init_pair();
        pair.set_base(&self.pair.base);
        pair.set_quote(&self.pair.quote);

        // Set timestamp_ms
        builder.set_timestamp_ms(self.timestamp_ms);

        // Set price (u128 to UInt128)
        let mut price_builder = builder.reborrow().init_price();
        price_builder.set_low(self.price as u64);
        price_builder.set_high((self.price >> 64) as u64);

        // Set volume (u128 to UInt128)
        let mut volume_builder = builder.reborrow().init_volume();
        volume_builder.set_low(self.volume as u64);
        volume_builder.set_high((self.volume >> 64) as u64);

        // Set expirationTimestamp union
        let mut expiration = builder.reborrow().init_expiration_timestamp();
        match self.expiration_timestamp {
            Some(ts) => {
                expiration.set_expiration_timestamp(ts);
            }
            None => {
                expiration.set_no_expiration(());
            }
        };

        let mut buffer = Vec::new();
        serialize::write_message(&mut buffer, &message).unwrap();
        buffer
    }
}

#[cfg(feature = "capnp")]
impl crate::CapnpDeserialize for PriceEntry {
    fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error>
    where
        Self: Sized,
    {
        let message_reader = serialize::read_message(bytes, capnp::message::ReaderOptions::new())?;
        let reader = message_reader.get_root::<schema_capnp::price_entry::Reader>()?;

        // Extract source
        let source = reader.get_source()?.to_string()?;

        // Extract chain from union
        let chain = match reader.get_chain().which()? {
            schema_capnp::price_entry::chain::NoChain(()) => None,
            schema_capnp::price_entry::chain::Chain(chain_reader) => Some(match chain_reader? {
                schema_capnp::Chain::Starknet => Chain::Starknet,
                schema_capnp::Chain::Solana => Chain::Solana,
                schema_capnp::Chain::Sui => Chain::Sui,
                schema_capnp::Chain::Aptos => Chain::Aptos,
                schema_capnp::Chain::Ethereum => Chain::Ethereum,
                schema_capnp::Chain::Base => Chain::Base,
                schema_capnp::Chain::Arbitrum => Chain::Arbitrum,
                schema_capnp::Chain::Optimism => Chain::Optimism,
                schema_capnp::Chain::Zksync => Chain::ZkSync,
                schema_capnp::Chain::Polygon => Chain::Polygon,
                schema_capnp::Chain::Bnb => Chain::Bnb,
                schema_capnp::Chain::Avalanche => Chain::Avalanche,
                schema_capnp::Chain::Gnosis => Chain::Gnosis,
                schema_capnp::Chain::Worldchain => Chain::Worldchain,
            }),
        };

        // Extract pair
        let pair_reader = reader.get_pair()?;
        let pair = Pair {
            base: pair_reader.get_base()?.to_string()?,
            quote: pair_reader.get_quote()?.to_string()?,
        };

        // Extract timestamp_ms
        let timestamp_ms = reader.get_timestamp_ms();

        // Extract price (UInt128 to u128)
        let price_reader = reader.get_price()?;
        let price = (price_reader.get_high() as u128) << 64 | price_reader.get_low() as u128;

        // Extract volume (UInt128 to u128)
        let volume_reader = reader.get_volume()?;
        let volume = (volume_reader.get_high() as u128) << 64 | volume_reader.get_low() as u128;

        // Extract expirationTimestamp from union
        let expiration_timestamp = match reader.get_expiration_timestamp().which()? {
            schema_capnp::price_entry::expiration_timestamp::NoExpiration(()) => None,
            schema_capnp::price_entry::expiration_timestamp::ExpirationTimestamp(ts) => Some(ts),
        };

        Ok(PriceEntry {
            source,
            chain,
            pair,
            timestamp_ms,
            price,
            volume,
            expiration_timestamp,
        })
    }
}
