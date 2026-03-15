//based on https://github.com/nrc/leb128/blob/master/src/lib.rs
pub fn encode_unsigned(val:u32) -> Vec<u8> {
    let mut val = val;
    let mut result = vec![];
    loop {
        let mut byte = val as u8 & 0b0111_1111;
        val >>= 7;
        if val != 0 {
            byte |= 0b1000_0000;
        }
        result.push(byte);


        if val == 0 {
            return result;
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use super::encode_unsigned;

    fn decode_leb128(bytes: &[u8]) -> u32 {
        let mut result = 0u32;
        let mut shift = 0;
        for byte in bytes {
            result |= ((byte & 0x7f) as u32) << shift;
            shift += 7;
        }
        result
    }

    proptest! {
        #[test]
        fn output_is_never_empty(val: u32) {
            prop_assert!(!encode_unsigned(val).is_empty());
        }

        #[test]
        fn last_byte_has_no_continuation_bit(val: u32) {
            let encoded = encode_unsigned(val);
            prop_assert_eq!(encoded.last().unwrap() & 0x80, 0);
        }

        #[test]
        fn all_non_last_bytes_have_continuation_bit(val: u32) {
            let encoded = encode_unsigned(val);
            let len = encoded.len();
            for &byte in encoded.iter().take(len.saturating_sub(1)) {
                prop_assert_eq!(byte & 0x80, 0x80);
            }
        }

        #[test]
        fn roundtrip_encoding(val: u32) {
            prop_assert_eq!(decode_leb128(&encode_unsigned(val)), val);
        }

        #[test]
        fn values_under_128_encode_as_single_byte(val in 0u32..128u32) {
            prop_assert_eq!(encode_unsigned(val).len(), 1);
        }

        #[test]
        fn values_128_and_above_need_multiple_bytes(val in 128u32..u32::MAX) {
            prop_assert!(encode_unsigned(val).len() > 1);
        }

        #[test]
        fn encoded_length_grows_with_value(
            small in 0u32..128u32,
            large in 128u32..16384u32
        ) {
            prop_assert!(encode_unsigned(small).len() <= encode_unsigned(large).len());
        }
    }
}
