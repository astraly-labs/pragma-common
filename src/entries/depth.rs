#[cfg(feature = "capnp")]
use capnp::serialize;

#[cfg(feature = "capnp")]
use crate::schema_capnp;
use crate::{instrument_type::InstrumentType, web3::Chain, Pair};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DepthEntry {
    pub source: String,
    pub chain: Option<Chain>,
    pub instrument_type: InstrumentType,
    pub pair: Pair,
    pub timestamp_ms: i64,
    pub depth: DepthLevel,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DepthLevel {
    pub percentage: f64,
    pub bid: f64,
    pub ask: f64,
}

#[cfg(feature = "capnp")]
impl crate::CapnpSerialize for DepthEntry {
    fn to_capnp(&self) -> Vec<u8> {
        let mut message = capnp::message::Builder::new_default();
        let mut builder = message.init_root::<schema_capnp::depth_entry::Builder>();

        builder.set_source(&self.source);
        builder.set_instrument_type(match self.instrument_type {
            InstrumentType::Spot => schema_capnp::InstrumentType::Spot,
            InstrumentType::Perp => schema_capnp::InstrumentType::Perp,
        });

        let mut pair = builder.reborrow().init_pair();
        pair.set_base(&self.pair.base);
        pair.set_quote(&self.pair.quote);

        let mut depth_level = builder.reborrow().init_depth();
        depth_level.set_percentage(self.depth.percentage);
        depth_level.set_bid(self.depth.bid);
        depth_level.set_ask(self.depth.ask);

        // Set the chain union
        let mut chain = builder.reborrow().init_chain();
        match &self.chain {
            Some(serialized_chain) => {
                chain.set_chain(match serialized_chain {
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

        // Set timestamp_ms
        builder.set_timestamp_ms(self.timestamp_ms);

        let mut buffer = Vec::new();
        serialize::write_message(&mut buffer, &message).unwrap();
        buffer
    }
}

#[cfg(feature = "capnp")]
impl crate::CapnpDeserialize for DepthEntry {
    fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error>
    where
        Self: Sized,
    {
        let message_reader = serialize::read_message(bytes, capnp::message::ReaderOptions::new())?;
        let reader = message_reader.get_root::<schema_capnp::depth_entry::Reader>()?;

        let source = reader.get_source()?.to_string()?;
        let instrument_type = match reader.get_instrument_type()? {
            schema_capnp::InstrumentType::Spot => InstrumentType::Spot,
            schema_capnp::InstrumentType::Perp => InstrumentType::Perp,
        };

        let pair_reader = reader.get_pair()?;
        let pair = Pair {
            base: pair_reader.get_base()?.to_string()?,
            quote: pair_reader.get_quote()?.to_string()?,
        };

        let depth_reader = reader.get_depth()?;
        let depth = DepthLevel {
            percentage: depth_reader.get_percentage(),
            bid: depth_reader.get_bid(),
            ask: depth_reader.get_ask(),
        };

        // Extract timestamp_ms
        let timestamp_ms = reader.get_timestamp_ms();

        // Extract chain from the union
        let chain = match reader.get_chain().which()? {
            schema_capnp::depth_entry::chain::NoChain(()) => None,
            schema_capnp::depth_entry::chain::Chain(chain_reader) => Some(match chain_reader? {
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

        Ok(DepthEntry {
            source,
            instrument_type,
            pair,
            depth,
            timestamp_ms,
            chain,
        })
    }
}
