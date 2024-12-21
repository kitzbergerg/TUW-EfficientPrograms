use std::{
    collections::VecDeque,
    simd::{Simd, cmp::SimdPartialEq, u8x64},
};

use crate::CsvField;

const CHUNK_SIZE: usize = 64;
const QUEUE_CAPACITY: usize = 128;

const SIMD_NEWLINE: Simd<u8, CHUNK_SIZE> = Simd::from_array([b'\n'; CHUNK_SIZE]);
const SIMD_COMMA: Simd<u8, CHUNK_SIZE> = Simd::from_array([b','; CHUNK_SIZE]);

pub struct SimdCsvReader<'a> {
    /// The remaining data to be processed
    data: &'a [u8],

    /// The resulting fields of the CSV
    result: VecDeque<&'a [u8]>,
}

impl<'a> SimdCsvReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        SimdCsvReader {
            data,
            result: VecDeque::with_capacity(QUEUE_CAPACITY),
        }
    }

    fn process_simd(&mut self, chunk: &'a [u8]) {
        let simd = u8x64::from_slice(chunk);
        let mask_comma = simd.simd_eq(SIMD_NEWLINE).to_bitmask();
        let mask_newline = simd.simd_eq(SIMD_COMMA).to_bitmask();

        // zeroes indicate a comma or newline
        let mut combined = mask_comma | mask_newline;

        let mut prev = 0;
        while combined != 0 {
            let i = combined.trailing_zeros() as usize;
            self.result.push_back(&self.data[prev..i]);
            prev = i + 1;
            combined -= 1 << i;
        }
        self.data = &self.data[prev..];
    }

    fn process_remainder(&mut self, remainder: &[u8]) {
        let mut prev = 0;
        remainder
            .iter()
            .enumerate()
            .filter(|(_, el)| *el == &b',' || *el == &b'\n')
            .for_each(|(i, _)| {
                self.result.push_back(&self.data[prev..i]);
                prev = i + 1;
            });
        self.data = &self.data[prev..];
    }

    fn fill_queue(&mut self) {
        while self.result.len() < QUEUE_CAPACITY && !self.data.is_empty() {
            if self.data.len() >= CHUNK_SIZE {
                self.process_simd(&self.data[..CHUNK_SIZE]);
            } else {
                self.process_remainder(self.data);
            }
        }
    }
}

impl<'a> Iterator for SimdCsvReader<'a> {
    type Item = (CsvField<'a>, CsvField<'a>);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.result.len() < 2 {
            self.fill_queue();
        }

        self.result
            .pop_front()
            .map(|field1| (field1, self.result.pop_front().unwrap()))
    }
}

pub trait IntoCsvReader<'a> {
    fn parse_csv(self) -> SimdCsvReader<'a>;
}

impl<'a> IntoCsvReader<'a> for &'a [u8] {
    fn parse_csv(self) -> SimdCsvReader<'a> {
        SimdCsvReader::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_reader() {
        let data = b"field1,value1\nf2,v2\nfield3,value3\n";
        let reader = data.as_slice().parse_csv();
        let results: Vec<_> = reader.collect();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], (b"field1".as_slice(), b"value1".as_slice()));
        assert_eq!(results[1], (b"f2".as_slice(), b"v2".as_slice()));
        assert_eq!(results[2], (b"field3".as_slice(), b"value3".as_slice()));
    }
}
