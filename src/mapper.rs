/// There are 37 unique bytes in each key. Keys are a max of 22 chars.
/// I can express each byte using 6 bits (since 2^6=64 > 37).
/// However 22*6=132, so it doesn't fit into u128.
///
/// Instead use pairs:
/// Since 37*37=1369 and 2^11=2048 we can compress byte pairs into 11 bits.
/// 11*11=121 which fits into u128.

const NUM_UNIQ: usize = 37;
const UNIQUE_BYTES: [u8; NUM_UNIQ] = [
    0, b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E',
    b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U',
    b'V', b'W', b'X', b'Y', b'Z',
];
const MAPPER_SINGLE: [u8; 256] = {
    let mut tmp = [0u8; 256];
    let mut count = 0;
    while count < UNIQUE_BYTES.len() {
        tmp[UNIQUE_BYTES[count] as usize] = count as u8;
        count += 1;
    }
    tmp
};

pub fn map_to_int(bytes: &[u8]) -> u128 {
    let len = bytes.len() as u128;
    let mut bytes = bytes;

    let mut acc = 0u128;
    while bytes.len() >= 2 {
        let idx1 = MAPPER_SINGLE[bytes[0] as usize] as usize;
        let idx2 = MAPPER_SINGLE[bytes[1] as usize] as usize;
        let merged = idx1 * NUM_UNIQ + idx2;
        acc <<= 11;
        acc |= merged as u128;
        bytes = &bytes[2..];
    }

    // In case the length is odd
    if bytes.len() == 1 {
        let last = bytes[0];
        acc <<= 11;
        acc |= MAPPER_SINGLE[last as usize] as u128;
    }

    // Store len in most significant bits
    acc | (len << 122)
}

const MASK: u128 = 0b111_1111_1111;
pub fn map_to_bytes(bytes: u128) -> (usize, [u8; 22]) {
    let mut bytes = bytes;
    let mut result = [0u8; 22];
    let len = (bytes >> 122) as usize;
    let mut i = len;

    // In case the length is odd
    if len % 2 == 1 {
        i -= 1;
        result[i] = UNIQUE_BYTES[(bytes & MASK) as usize];
        bytes >>= 11;
    }
    for _ in 0..len / 2 {
        let num = bytes & MASK;
        result[i - 1] = UNIQUE_BYTES[num as usize % NUM_UNIQ];
        result[i - 2] = UNIQUE_BYTES[num as usize / NUM_UNIQ];
        bytes >>= 11;
        i -= 2;
    }

    (len, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapper_odd() {
        let bytes = "0".as_bytes();
        assert!(bytes.len() % 2 == 1);
        let mapped = map_to_bytes(map_to_int(bytes));
        assert_eq!(bytes, &mapped.1[0..mapped.0]);

        let bytes = "1H4YIF8Z6VD5ALVBZ".as_bytes();
        assert!(bytes.len() % 2 == 1);
        let mapped = map_to_bytes(map_to_int(bytes));
        assert_eq!(bytes, &mapped.1[0..mapped.0]);
    }

    #[test]
    fn test_mapper_even() {
        let bytes = "00".as_bytes();
        assert!(bytes.len() % 2 == 0);
        let mapped = map_to_bytes(map_to_int(bytes));
        assert_eq!(bytes, &mapped.1[0..mapped.0]);

        let bytes = "AREO43YITEGS3DT3UZ".as_bytes();
        assert!(bytes.len() % 2 == 0);
        let mapped = map_to_bytes(map_to_int(bytes));
        assert_eq!(bytes, &mapped.1[0..mapped.0]);
    }
}
