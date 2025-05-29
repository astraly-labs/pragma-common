use starknet::core::types::Felt;

pub mod starknet_felt_conversion {
    use super::Felt; // Import Felt from the parent module (or crate)

    pub fn felt_to_string(num: &Felt) -> String {
        let bytes = num.to_bytes_be();
        let trimmed = bytes
            .iter()
            .skip_while(|b| **b == 0)
            .cloned()
            .collect::<Vec<u8>>();

        String::from_utf8(trimmed).unwrap_or_else(|_| "<invalid UTF-8>".to_string())
    }

    pub fn felt_to_u128(felt: &Felt) -> Option<u128> {
        let bytes = felt.to_bytes_be();

        let len = bytes.len();
        let start = len.saturating_sub(16);
        let slice = &bytes[start..];

        let mut buf = [0u8; 16];
        // Ensure slice is not longer than 16 bytes before copying
        let copy_len = std::cmp::min(slice.len(), 16);
        let slice_start = slice.len() - copy_len; // take from the end of the slice
        buf[(16 - copy_len)..].copy_from_slice(&slice[slice_start..]);

        Some(u128::from_be_bytes(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::starknet_felt_conversion::*; // Use the new module
    use starknet::core::types::Felt;

    // Helper function to create Felt from a byte slice, padding/truncating to 32 bytes BE
    fn felt_from_custom_bytes(slice: &[u8]) -> Felt {
        let mut felt_buffer = [0u8; 32];
        let len = slice.len();
        if len == 0 {
            return Felt::ZERO;
        }
        let copy_len = std::cmp::min(len, 32);
        let src_start = len - copy_len;
        let dest_start = 32 - copy_len;
        felt_buffer[dest_start..dest_start + copy_len]
            .copy_from_slice(&slice[src_start..src_start + copy_len]);
        Felt::from_bytes_be(&felt_buffer)
    }

    #[test]
    fn test_felt_to_string_simple() {
        let felt = Felt::from_hex_unchecked("0x68656c6c6f");
        assert_eq!(felt_to_string(&felt), "hello");
    }

    #[test]
    fn test_felt_to_string_leading_zeros_in_felt() {
        let felt = Felt::from_hex_unchecked("0x616263");
        assert_eq!(felt_to_string(&felt), "abc");
    }

    #[test]
    fn test_felt_to_string_empty_after_trimming() {
        let felt = Felt::ZERO;
        assert_eq!(felt_to_string(&felt), "");
    }

    #[test]
    fn test_felt_to_string_max_felt_as_string() {
        let felt = Felt::MAX;
        assert_eq!(felt_to_string(&felt), "\x08\x00\x00\x00\x00\x00\x00\x11\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00");
    }

    #[test]
    fn test_felt_to_string_invalid_utf8() {
        let bytes = [0xC0, 0x80];
        let felt = felt_from_custom_bytes(&bytes);
        assert_eq!(felt_to_string(&felt), "<invalid UTF-8>");
    }

    #[test]
    fn test_felt_to_u128_simple_value() {
        let felt = Felt::from_dec_str("12345").unwrap();
        assert_eq!(felt_to_u128(&felt), Some(12345u128));
    }

    #[test]
    fn test_felt_to_u128_zero() {
        let felt = Felt::ZERO;
        assert_eq!(felt_to_u128(&felt), Some(0u128));
    }

    #[test]
    fn test_felt_to_u128_max_u128() {
        let max_u128 = u128::MAX;
        let felt = felt_from_custom_bytes(&max_u128.to_be_bytes());
        assert_eq!(felt_to_u128(&felt), Some(max_u128));
    }

    #[test]
    fn test_felt_to_u128_slightly_less_than_max_u128() {
        let val = u128::MAX - 1;
        let felt = felt_from_custom_bytes(&val.to_be_bytes());
        assert_eq!(felt_to_u128(&felt), Some(val));
    }

    #[test]
    fn test_felt_to_u128_value_with_leading_zeros_in_felt_bytes() {
        let felt = Felt::from_dec_str("255").unwrap();
        assert_eq!(felt_to_u128(&felt), Some(255u128));
    }

    #[test]
    fn test_felt_to_u128_too_large() {
        let felt_from_overflow_bytes =
            Felt::from_hex_unchecked("0x100000000000000000000000000000000"); // 2^128
        assert_eq!(felt_to_u128(&felt_from_overflow_bytes), Some(0u128));

        let felt_max_val = Felt::MAX;
        let felt_max_bytes = felt_max_val.to_bytes_be();
        let mut expected_bytes_arr = [0u8; 16];
        expected_bytes_arr.copy_from_slice(&felt_max_bytes[felt_max_bytes.len() - 16..]);
        let expected_u128 = u128::from_be_bytes(expected_bytes_arr);
        assert_eq!(felt_to_u128(&felt_max_val), Some(expected_u128));
    }

    #[test]
    fn test_felt_to_u128_1byte() {
        let felt = Felt::from_dec_str("1").unwrap();
        assert_eq!(felt_to_u128(&felt), Some(1u128));
    }

    #[test]
    fn test_felt_to_u128_15bytes() {
        let val = (1u128 << (8 * 15 - 1)) - 1;
        let felt = felt_from_custom_bytes(&val.to_be_bytes()[1..]);
        assert_eq!(felt_to_u128(&felt), Some(val));
    }

    #[test]
    fn test_felt_to_u128_16bytes() {
        let val = (1u128 << (8 * 16 - 1)) - 1;
        let felt = felt_from_custom_bytes(&val.to_be_bytes());
        assert_eq!(felt_to_u128(&felt), Some(val));
    }
}
