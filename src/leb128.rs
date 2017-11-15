//based on https://github.com/nrc/leb128/blob/master/src/lib.rs

pub fn encode_signed(val:i32) -> Vec<u8> {
    let mut val = val;
    const SIGN_BIT: u8 = 0b0100_0000;
    let mut result = vec![];
    let mut more = true;
    loop {
        let mut byte = val as u8 & 0b0111_1111;
        val >>= 7;
        if (val == 0 && byte & SIGN_BIT == 0) ||
            (val == -1 && byte & SIGN_BIT != 0) {
            more = false;
        } else {
            byte |= 0b1000_0000;
        }
        result.push(byte);

        if !more {
            return result;
        }
    }
}

pub fn encode_unsigned(val:u32) -> Vec<u8> {
    let mut val = val;
    const SIGN_BIT: u8 = 0b0100_0000;
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

