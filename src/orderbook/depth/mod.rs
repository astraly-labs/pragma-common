#[cfg(feature = "capnp")]
use crate::schema_capnp::{self, depth};
use crate::{web3::Chain, InstrumentType, Pair};
#[cfg(feature = "capnp")]
use capnp::serialize;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Depth {
    pub depth: DepthLevel,
    pub pair: Pair,
    pub source: String,
    pub chain: Option<Chain>,
    pub instrument_type: InstrumentType,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepthLevel {
    pub percentage: f64,
    pub bid: f64,
    pub ask: f64,
}

#[cfg(feature = "capnp")]
impl crate::schema_capnp::CapnpSerialize for Depth {
    fn to_capnp(&self) -> Vec<u8> {
        let mut message = capnp::message::Builder::new_default();
        let mut builder = message.init_root::<depth::Builder>();

        let mut depth_level = builder.reborrow().init_depth();
        depth_level.set_percentage(self.depth.percentage);
        depth_level.set_bid(self.depth.bid);
        depth_level.set_ask(self.depth.ask);

        let mut pair = builder.reborrow().init_pair();
        pair.set_base(&self.pair.base);
        pair.set_quote(&self.pair.quote);

        builder.set_source(&self.source);
        builder.set_instrument_type(match self.instrument_type {
            InstrumentType::Spot => schema_capnp::InstrumentType::Spot,
            InstrumentType::Perp => schema_capnp::InstrumentType::Perp,
        });

        match &self.chain {
            Some(chain) => builder.reborrow().set_chain(match chain {
                Chain::Ethereum => schema_capnp::Chain::Ethereum,
                Chain::Starknet => schema_capnp::Chain::Starknet,
                Chain::Solana => schema_capnp::Chain::Solana,
                Chain::Sui => schema_capnp::Chain::Sui,
                Chain::Aptos => schema_capnp::Chain::Aptos,
                Chain::Base => schema_capnp::Chain::Base,
                Chain::Arbitrum => schema_capnp::Chain::Arbitrum,
                Chain::Optimism => schema_capnp::Chain::Optimism,
                Chain::ZkSync => schema_capnp::Chain::Zksync,
                Chain::Polygon => schema_capnp::Chain::Polygon,
                Chain::Bnb => schema_capnp::Chain::Bnb,
                Chain::Avalanche => schema_capnp::Chain::Avalanche,
                Chain::Gnosis => schema_capnp::Chain::Gnosis,
                Chain::Worldchain => schema_capnp::Chain::Worldchain,
            }),
            None => builder.reborrow().set_no_chain(()),
        };

        let mut buffer = Vec::new();
        serialize::write_message(&mut buffer, &message).unwrap();
        buffer
    }
}

#[cfg(feature = "capnp")]
impl crate::schema_capnp::CapnpDeserialize for Depth {
    fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error>
    where
        Self: Sized,
    {
        let message_reader = serialize::read_message(bytes, capnp::message::ReaderOptions::new())?;
        let reader = message_reader.get_root::<depth::Reader>()?;

        // Extract depth
        let depth_reader = reader.get_depth()?;
        let depth = DepthLevel {
            percentage: depth_reader.get_percentage(),
            bid: depth_reader.get_bid(),
            ask: depth_reader.get_ask(),
        };

        // Extract pair
        let pair_reader = reader.get_pair()?;
        let pair = Pair {
            base: pair_reader.get_base()?.to_string()?,
            quote: pair_reader.get_quote()?.to_string()?,
        };

        // Extract source
        let source = reader.get_source()?.to_string()?;

        // Extract instrument_type
        let instrument_type = match reader.get_instrument_type()? {
            schema_capnp::InstrumentType::Spot => InstrumentType::Spot,
            schema_capnp::InstrumentType::Perp => InstrumentType::Perp,
        };

        // Extract chain from the union
        let chain = match reader.which()? {
            depth::Which::NoChain(()) => None,
            depth::Which::Chain(chain_reader) => Some(match chain_reader? {
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

        Ok(Depth {
            depth,
            pair,
            source,
            chain,
            instrument_type,
        })
    }
}
