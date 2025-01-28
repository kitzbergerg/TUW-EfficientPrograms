use std::simd::{Simd, cmp::SimdPartialEq, u8x64};

use crate::Field;

const CHUNK_SIZE: usize = 64;

const SIMD_NEWLINE: Simd<u8, CHUNK_SIZE> = Simd::from_array([b'\n'; CHUNK_SIZE]);
const SIMD_COMMA: Simd<u8, CHUNK_SIZE> = Simd::from_array([b','; CHUNK_SIZE]);

pub fn parse_csv(data: &[u8]) -> Vec<(Field<'_>, Field<'_>)> {
    let mut fields: Vec<Field<'_>> = Vec::with_capacity(data.len() / 8);
    let mut pos = 0;
    let mut prev = 0;
    while pos + CHUNK_SIZE < data.len() {
        let simd = u8x64::from_slice(unsafe { data.get_unchecked(pos..pos + CHUNK_SIZE) });
        let combined = simd.simd_eq(SIMD_NEWLINE) | simd.simd_eq(SIMD_COMMA);
        let combined = combined.to_bitmask();

        find_indices(data, &mut fields, &mut prev, pos, combined);

        pos += CHUNK_SIZE;
    }
    data[pos..]
        .iter()
        .enumerate()
        .filter(|(_, el)| *el == &b',' || *el == &b'\n')
        .for_each(|(i, _)| {
            let field = unsafe { data.get_unchecked(prev..pos + i) };
            fields.push(field);
            prev = pos + i + 1;
        });

    let mut pairs = Vec::with_capacity(fields.len() / 2 + 1);
    let mut iter = fields.into_iter();
    while let (Some(f1), Some(f2)) = (iter.next(), iter.next()) {
        pairs.push((f1, f2));
    }

    pairs
}

#[cfg(not(target_feature = "avx512f"))]
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
        let current_pos = pos + i;
        let field = unsafe { data.get_unchecked(*prev..current_pos) };
        fields.push(field);
        *prev = current_pos + 1;
        combined -= 1 << i;
    }
}

#[cfg(target_feature = "avx512f")]
#[inline(always)]
fn find_indices<'a>(
    data: &'a [u8],
    fields: &mut Vec<Field<'a>>,
    prev: &mut usize,
    pos: usize,
    combined: u64,
) {
    const RANGE: Simd<u8, CHUNK_SIZE> = {
        let mut tmp = [0u8; CHUNK_SIZE];
        let mut i = 0u8;
        while i < CHUNK_SIZE as u8 {
            tmp[i as usize] = i;
            i += 1;
        }
        Simd::from_array(tmp)
    };

    let mut offsets = [0u8; CHUNK_SIZE];
    unsafe {
        let hits = std::arch::x86_64::_mm512_maskz_mov_epi8(combined, RANGE.into());
        std::arch::x86_64::_mm512_mask_compressstoreu_epi8(offsets.as_mut_ptr(), combined, hits);
    }
    for i in 0..combined.count_ones() {
        let i = offsets[i as usize] as usize;
        let current_pos = pos + i;
        let field = unsafe { data.get_unchecked(*prev..current_pos) };
        fields.push(field);
        *prev = current_pos + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_reader() {
        let data = b"field1,value1\nfield2,value2\nfield3,value3\n";
        let results = parse_csv(data.as_slice());

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], (b"field1".as_slice(), b"value1".as_slice()));
        assert_eq!(results[1], (b"field2".as_slice(), b"value2".as_slice()));
        assert_eq!(results[2], (b"field3".as_slice(), b"value3".as_slice()));
    }
}
