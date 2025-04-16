#[cfg(feature = "capnp")]
use capnp::serialize;

#[cfg(feature = "capnp")]
use crate::schema_capnp;
use crate::Pair;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FundingRateEntry {
    pub source: String,
    pub pair: Pair,
    pub funding_rate: f64,
    pub timestamp_ms: i64,
}

#[cfg(feature = "capnp")]
impl crate::CapnpSerialize for FundingRateEntry {
    fn to_capnp(&self) -> Vec<u8> {
        let mut message = capnp::message::Builder::new_default();
        let mut builder = message.init_root::<schema_capnp::funding_rate_entry::Builder>();

        // Set source
        builder.set_source(&self.source);

        // Set pair
        let mut pair = builder.reborrow().init_pair();
        pair.set_base(&self.pair.base);
        pair.set_quote(&self.pair.quote);

        // Set funding rate
        builder.set_funding_rate(self.funding_rate);

        // Set timestamp_ms
        builder.set_timestamp_ms(self.timestamp_ms);

        let mut buffer = Vec::new();
        serialize::write_message(&mut buffer, &message).unwrap();
        buffer
    }
}

#[cfg(feature = "capnp")]
impl crate::CapnpDeserialize for FundingRateEntry {
    fn from_capnp(bytes: &[u8]) -> Result<Self, capnp::Error>
    where
        Self: Sized,
    {
        let message_reader = serialize::read_message(bytes, capnp::message::ReaderOptions::new())?;
        let reader = message_reader.get_root::<schema_capnp::funding_rate_entry::Reader>()?;

        let source = reader.get_source()?.to_string()?;

        let funding_rate = reader.get_funding_rate();

        let pair_reader = reader.get_pair()?;
        let pair = Pair {
            base: pair_reader.get_base()?.to_string()?,
            quote: pair_reader.get_quote()?.to_string()?,
        };

        // Extract timestamp_ms
        let timestamp_ms = reader.get_timestamp_ms();

        Ok(FundingRateEntry {
            source,
            pair,
            funding_rate,
            timestamp_ms,
        })
    }
}
