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

