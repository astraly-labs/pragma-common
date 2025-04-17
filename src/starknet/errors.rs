use starknet::core::crypto::EcdsaSignError;
use starknet::core::types::Felt;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub enum ConversionError {
    #[error("failed to serialize")]
    FailedSerialization,
    #[error("invalid date time")]
    InvalidDateTime,
    #[error("failed to convert big decimal")]
    BigDecimalConversion,
    #[error("failed to convert felt")]
    FeltConversion,
    #[error("failed to convert u128")]
    U128Conversion,
    #[error("failed to convert timestamp string")]
    StringTimestampConversion,
    #[error("failed to convert price string")]
    StringPriceConversion,
    #[error("fail to sign for pair {0:?}")]
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    FailedSignature(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum SignerError {
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error("cannot sign: {0}")]
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    SigningError(#[from] EcdsaSignError),
    #[error("invalid signature for message hash {0:?}")]
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    InvalidSignature(Felt),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("invalid message: {0}")]
    InvalidMessage(String),
}
