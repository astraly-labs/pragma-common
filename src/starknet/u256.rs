use num_bigint::BigUint;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use starknet::core::types::Felt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StarknetU256 {
    pub low: Felt,
    pub high: Felt,
}

impl StarknetU256 {
    pub const ZERO: StarknetU256 = StarknetU256 {
        low: Felt::ZERO,
        high: Felt::ZERO,
    };
}

#[derive(Debug, thiserror::Error)]
#[error("Slice too long, max is 32, received {0}")]
pub struct StarknetU256FromBytesSliceError(usize);

impl StarknetU256 {
    pub fn from_parts<L: Into<u128>, H: Into<u128>>(low: L, high: H) -> Self {
        let low: u128 = low.into();
        let high: u128 = high.into();
        Self {
            low: Felt::from(low),
            high: Felt::from(high),
        }
    }

    pub fn to_bytes_be(&self) -> [u8; 32] {
        let mut ret = self.low.to_bytes_be();

        ret[0..16].copy_from_slice(&self.high.to_bytes_be()[16..32]);

        ret
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            low: Felt::from(u128::from_be_bytes(bytes[16..].try_into().unwrap())),
            high: Felt::from(u128::from_be_bytes(bytes[..16].try_into().unwrap())),
        }
    }

    pub fn from_bytes_slice(bytes: &[u8]) -> Result<Self, StarknetU256FromBytesSliceError> {
        let (low, high) = match bytes.len() {
            0 => return Ok(Self::ZERO),
            16 => (u128::from_be_bytes(bytes.try_into().unwrap()), 0u128),
            32 => (
                u128::from_be_bytes(bytes[16..32].try_into().unwrap()),
                u128::from_be_bytes(bytes[0..16].try_into().unwrap()),
            ),
            l if l < 16 => {
                let mut low = [0u8; 16];
                low[16 - l..].copy_from_slice(bytes);
                (u128::from_be_bytes(low), 0u128)
            }
            l if l < 32 => {
                let mut low_bytes = [0u8; 16];
                low_bytes.copy_from_slice(&bytes[l - 16..]);

                let mut high_bytes = [0u8; 16];
                let high_part_len = l - 16;
                high_bytes[16 - high_part_len..].copy_from_slice(&bytes[..high_part_len]);

                (
                    u128::from_be_bytes(low_bytes),
                    u128::from_be_bytes(high_bytes),
                )
            }
            l => return Err(StarknetU256FromBytesSliceError(l)),
        };

        Ok(Self::from_parts(low, high))
    }
}

impl core::fmt::Display for StarknetU256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "low: {:#x} - high: {:#x}", self.low, self.high)
    }
}

impl From<StarknetU256> for BigUint {
    fn from(value: StarknetU256) -> Self {
        let bytes = value.to_bytes_be();
        BigUint::from_bytes_be(&bytes)
    }
}

impl From<&StarknetU256> for BigUint {
    fn from(value: &StarknetU256) -> Self {
        let bytes = value.to_bytes_be();
        BigUint::from_bytes_be(&bytes)
    }
}

impl From<StarknetU256> for Decimal {
    fn from(value: StarknetU256) -> Self {
        let biguint_representation: BigUint = value.into();
        let scaled_decimal_value = Decimal::from_str(&biguint_representation.to_string())
            .expect("BigUint::to_string should always yield a valid number for Decimal parsing");
        scaled_decimal_value
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TryU256FromBigUintError {
    #[error("BigUint too big")]
    TooBig,
}

impl TryFrom<BigUint> for StarknetU256 {
    type Error = TryU256FromBigUintError;

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes_le();
        if bytes.len() > 32 {
            return Err(Self::Error::TooBig);
        };

        if bytes.len() < 16 {
            return Ok(StarknetU256 {
                low: Felt::from_bytes_le_slice(&bytes),
                high: Felt::ZERO,
            });
        }

        Ok(StarknetU256 {
            low: Felt::from_bytes_le_slice(&bytes[0..16]),
            high: Felt::from_bytes_le_slice(&bytes[16..]),
        })
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use starknet::core::types::Felt;

    use super::{StarknetU256, StarknetU256FromBytesSliceError};

    #[test]
    fn test_zero() {
        let zero = StarknetU256::ZERO;
        assert_eq!(zero.low, Felt::ZERO);
        assert_eq!(zero.high, Felt::ZERO);
    }

    #[test]
    fn test_from_parts() {
        let value = StarknetU256::from_parts(42u64, 0u64);
        assert_eq!(value.low, Felt::from(42u128));
        assert_eq!(value.high, Felt::ZERO);

        let value = StarknetU256::from_parts(u128::MAX, 1u64);
        assert_eq!(value.low, Felt::from(u128::MAX));
        assert_eq!(value.high, Felt::from(1u128));
    }

    #[test]
    fn test_to_bytes_be() {
        let value = StarknetU256::from_parts(0x1234567890ABCDEFu64, 0xFEDCBA0987654321u64);
        let bytes = value.to_bytes_be();

        // Verify the bytes are in big-endian order
        assert_eq!(
            bytes[24..],
            [0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]
        );
        assert_eq!(
            bytes[8..16],
            [0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65, 0x43, 0x21]
        );
    }

    #[test]
    fn test_from_bytes() {
        let mut bytes = [0u8; 32];
        bytes[16..24].copy_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]);
        bytes[0..8].copy_from_slice(&[0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65, 0x43, 0x21]);

        let value = StarknetU256::from_bytes(&bytes);
        assert_eq!(
            value.low,
            Felt::from(0x1234567890abcdef0000000000000000_u128)
        );
        assert_eq!(
            value.high,
            Felt::from(0xFEDCBA09876543210000000000000000_u128)
        );
    }

    #[test]
    fn test_from_bytes_slice() {
        // Test empty slice
        assert_eq!(
            StarknetU256::from_bytes_slice(&[]).unwrap(),
            StarknetU256::ZERO
        );

        // Test 16-byte slice
        let bytes = [
            0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB,
            0xCD, 0xEF,
        ];
        let value = StarknetU256::from_bytes_slice(&bytes).unwrap();
        assert_eq!(value.low, Felt::from(u128::from_be_bytes(bytes)));
        assert_eq!(value.high, Felt::ZERO);

        // Test 32-byte slice
        let mut bytes = [0u8; 32];
        // Low part (last 16 bytes) - only set first 8 bytes
        bytes[24..].copy_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]);
        // High part (first 16 bytes) - set first 8 bytes, pad with zeros
        bytes[8..16].copy_from_slice(&[0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65, 0x43, 0x21]);
        let value = StarknetU256::from_bytes_slice(&bytes).unwrap();
        assert_eq!(value.low, Felt::from(0x1234567890ABCDEF_u128));
        assert_eq!(value.high, Felt::from(0xFEDCBA0987654321_u128));

        // Test slice too long
        let bytes = [0u8; 33];
        assert!(matches!(
            StarknetU256::from_bytes_slice(&bytes),
            Err(StarknetU256FromBytesSliceError(33))
        ));
    }

    #[test]
    fn test_try_from_biguint() {
        // Test small number
        let biguint = BigUint::from(42u64);
        let value = StarknetU256::try_from(biguint).unwrap();
        assert_eq!(value.low, Felt::from(42u128));
        assert_eq!(value.high, Felt::ZERO);

        // Test number that fits in low part
        let biguint = BigUint::from(u128::MAX);
        let value = StarknetU256::try_from(biguint).unwrap();
        assert_eq!(value.low, Felt::from(u128::MAX));
        assert_eq!(value.high, Felt::ZERO);

        // Test number that needs both parts
        let biguint = BigUint::from(u128::MAX) + BigUint::from(1u64);
        let value = StarknetU256::try_from(biguint).unwrap();
        assert_eq!(value.low, Felt::from(0u128));
        assert_eq!(value.high, Felt::from(1u128));

        let mut bytes = [0u8; 33];
        bytes[32] = 1;
        let biguint = BigUint::from_bytes_le(&bytes);
        assert!(matches!(
            StarknetU256::try_from(biguint),
            Err(super::TryU256FromBigUintError::TooBig)
        ));
    }

    #[test]
    fn test_display() {
        let value = StarknetU256 {
            low: Felt::from_hex_unchecked("0x1234"),
            high: Felt::from_hex_unchecked("0x5678"),
        };
        let display = format!("{}", value);
        assert!(display.contains("low: 0x1234"));
        assert!(display.contains("high: 0x5678"));
    }
}
