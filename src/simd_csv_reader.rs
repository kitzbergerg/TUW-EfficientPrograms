use std::{
    collections::VecDeque,
    simd::{cmp::SimdPartialEq, u8x64, LaneCount, Mask, Simd, SupportedLaneCount},
    slice::ChunksExact,
};

use crate::CsvField;

const CHUNK_SIZE: usize = 64;
const QUEUE_CAPACITY: usize = 64;

pub struct SimdCsvReader<'a> {
    data: &'a [u8],
    chunks: ChunksExact<'a, u8>,
    i: usize,
    done: bool,

    simd_newline: Simd<u8, CHUNK_SIZE>,
    simd_comma: Simd<u8, CHUNK_SIZE>,

    idx_newline: VecDeque<usize>,
    idx_comma: VecDeque<usize>,
    prev: usize,
}

impl<'a> SimdCsvReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        SimdCsvReader {
            data,
            chunks: data.chunks_exact(CHUNK_SIZE),
            i: 0,
            done: false,
            simd_newline: Simd::splat(b'\n'),
            simd_comma: Simd::splat(b','),
            idx_newline: VecDeque::with_capacity(QUEUE_CAPACITY),
            idx_comma: VecDeque::with_capacity(QUEUE_CAPACITY),
            prev: 0,
        }
    }

    fn get_indices<const N: usize>(mask: &mut Mask<i8, N>, result: &mut VecDeque<usize>, i: usize)
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let num_matches = mask.to_bitmask().count_ones() as usize;
        for _ in 0..num_matches {
            let idx = mask.to_bitmask().trailing_zeros() as usize;
            result.push_back(i * CHUNK_SIZE + idx);
            mask.set(idx, false);
        }
    }

    fn find_next_delimiter(
        chunk: &'a [u8],
        splat: Simd<u8, CHUNK_SIZE>,
        result: &mut VecDeque<usize>,
        i: usize,
    ) {
        let mut mask = u8x64::from_slice(chunk).simd_eq(splat);
        SimdCsvReader::get_indices(&mut mask, result, i)
    }

    fn process_remainder(remainder: &[u8], delimiter: u8, result: &mut VecDeque<usize>, i: usize) {
        remainder
            .iter()
            .enumerate()
            .filter(|(_, &el)| el == delimiter)
            .for_each(|(idx, _)| result.push_back(i * CHUNK_SIZE + idx));
    }

    fn fill_queue(&mut self) {
        while self.idx_newline.len() < QUEUE_CAPACITY {
            if let Some(chunk) = self.chunks.next() {
                SimdCsvReader::find_next_delimiter(
                    chunk,
                    self.simd_comma,
                    &mut self.idx_comma,
                    self.i,
                );
                SimdCsvReader::find_next_delimiter(
                    chunk,
                    self.simd_newline,
                    &mut self.idx_newline,
                    self.i,
                );
                self.i += 1;
            } else if !self.done {
                let remainder = self.chunks.remainder();
                Self::process_remainder(remainder, b',', &mut self.idx_comma, self.i);
                Self::process_remainder(remainder, b'\n', &mut self.idx_newline, self.i);
                self.done = true;
            } else {
                return;
            }
        }
    }
}

impl<'a> Iterator for SimdCsvReader<'a> {
    type Item = (CsvField<'a>, CsvField<'a>);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx_newline.is_empty() {
            self.fill_queue();
        }

        // check for newline first, otherwise it might fails
        if let (Some(newline), Some(comma)) =
            (self.idx_newline.pop_front(), self.idx_comma.pop_front())
        {
            let result = (&self.data[self.prev..comma], &self.data[comma + 1..newline]);
            self.prev = newline + 1;
            Some(result)
        } else {
            None
        }
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
