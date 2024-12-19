use std::{
    io::Write,
    simd::{cmp::SimdPartialEq, Simd},
};

#[repr(align(32))]
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct CsvField([u8; 32]);

impl CsvField {
    #[inline(always)]
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut field = Self([b' '; 32]);
        field.0[..slice.len()].copy_from_slice(slice);
        field
    }

    #[inline(always)]
    pub fn write_trimmed<W: Write>(&self, writer: &mut W) {
        // Use SIMD for comparison but keep data as bytes
        let v: Simd<u8, 32> = Simd::from_slice(&self.0);
        let space = Simd::splat(b' ');
        let mask = v.simd_ne(space);

        let last_idx = if mask.any() {
            mask.to_bitmask().trailing_ones() as usize
        } else {
            1
        };

        writer.write_all(&self.0[..last_idx]).unwrap();
    }
}
