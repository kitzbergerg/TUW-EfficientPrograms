use std::simd::{Mask, Simd, cmp::SimdPartialEq, u8x64};

use crate::Field;

const CHUNK_SIZE: usize = 64;

const SIMD_NEWLINE: Simd<u8, CHUNK_SIZE> = Simd::from_array([b'\n'; CHUNK_SIZE]);
const SIMD_COMMA: Simd<u8, CHUNK_SIZE> = Simd::from_array([b','; CHUNK_SIZE]);

pub fn parse_csv<'a>(data: &'a [u8], fields: &mut Vec<Field<'a>>) {
    let mut prev = 0;
    let mut pos = 0;
    data.array_chunks()
        .map(|chunk| u8x64::from_array(*chunk))
        .map(|chunk| chunk.simd_eq(SIMD_NEWLINE) | chunk.simd_eq(SIMD_COMMA))
        .map(Mask::to_bitmask)
        .for_each(|mask| {
            find_indices(data, fields, &mut prev, pos, mask);
            pos += CHUNK_SIZE
        });
    data[pos..]
        .iter()
        .enumerate()
        .filter(|(_, el)| *el == &b',' || *el == &b'\n')
        .for_each(|(i, _)| {
            let current = pos + i;
            let field = unsafe { data.get_unchecked(prev..current) };
            fields.push(field);
            prev = current + 1;
        });
}

#[cfg(not(target_feature = "avx512vbmi2"))]
#[inline(always)]
fn find_indices<'a>(
    data: &'a [u8],
    fields: &mut Vec<Field<'a>>,
    prev: &mut usize,
    pos: usize,
    mut combined: u64,
) {
    while combined != 0 {
        let i = combined.trailing_zeros() as usize;
        let current = pos + i;
        let field = unsafe { data.get_unchecked(*prev..current) };
        fields.push(field);
        *prev = current + 1;
        combined &= combined - 1;
    }
}

#[cfg(target_feature = "avx512vbmi2")]
#[inline(always)]
fn find_indices<'a>(
    data: &'a [u8],
    fields: &mut Vec<Field<'a>>,
    prev: &mut usize,
    pos: usize,
    combined: u64,
) {
    const RANGE: u8x64 = {
        let mut tmp = [0u8; CHUNK_SIZE];
        let mut i = 0u8;
        while i < CHUNK_SIZE as u8 {
            tmp[i as usize] = i;
            i += 1;
        }
        Simd::from_array(tmp)
    };

    let offsets: u8x64 =
        unsafe { std::arch::x86_64::_mm512_maskz_compress_epi8(combined, RANGE.into()) }.into();
    for i in 0..combined.count_ones() {
        let i = offsets[i as usize] as usize;
        let current = pos + i;
        let field = unsafe { data.get_unchecked(*prev..current) };
        fields.push(field);
        *prev = current + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_reader() {
        let data = b"field1,value1\nfield2,value2\nfield3,value3\n";
        let mut results = Vec::new();
        parse_csv(data.as_slice(), &mut results);

        assert_eq!(results.len(), 6);
        assert_eq!(results[0], b"field1".as_slice());
        assert_eq!(results[1], b"value1".as_slice());
        assert_eq!(results[2], b"field2".as_slice());
        assert_eq!(results[3], b"value2".as_slice());
        assert_eq!(results[4], b"field3".as_slice());
        assert_eq!(results[5], b"value3".as_slice());
    }
}
