use std::{
    cmp::min,
    collections::VecDeque,
    simd::{cmp::SimdPartialEq, u8x64, LaneCount, Mask, Simd, SupportedLaneCount},
};

use crate::CsvField;

const CHUNK_SIZE: usize = 64;
const QUEUE_CAPACITY: usize = 64;

// Keys have length >4 so there cannot be more than (CHUNK_SIZE / 10) matches.
// 2*4+1+1=10 meaning 2 keys per line, 1 comma and 1 newline
const IDX_ARRAY_LEN: usize = (CHUNK_SIZE / 10) + 1;
type IdxArray = [usize; IDX_ARRAY_LEN];

pub struct SimdCsvReader<'a> {
    queue: VecDeque<CsvField<'a>>,
    data: &'a [u8],
    current_pos: usize,

    simd_newline: Simd<u8, CHUNK_SIZE>,
    simd_comma: Simd<u8, CHUNK_SIZE>,

    idx_newline: IdxArray,
    idx_comma: IdxArray,
}

impl<'a> SimdCsvReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        SimdCsvReader {
            queue: VecDeque::with_capacity(QUEUE_CAPACITY),
            data,
            current_pos: 0,
            simd_newline: Simd::splat(b'\n'),
            simd_comma: Simd::splat(b','),
            idx_newline: [0; IDX_ARRAY_LEN],
            idx_comma: [0; IDX_ARRAY_LEN],
        }
    }

    #[inline(always)]
    fn get_indices<const N: usize>(
        mask: &mut Mask<i8, N>,
        start: usize,
        result: &mut IdxArray,
    ) -> usize
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let num_matches = mask.to_bitmask().count_ones() as usize;
        for i in 0..num_matches {
            let idx = mask.to_bitmask().trailing_zeros() as usize;
            result[i] = start + idx;
            mask.set(idx, false);
        }
        num_matches
    }

    fn find_next_delimiter(
        data: &'a [u8],
        start: usize,
        splat: Simd<u8, CHUNK_SIZE>,
        delimiter: u8,
        result: &mut IdxArray,
    ) -> usize {
        let remaining = &data[start..];
        if remaining.len() < CHUNK_SIZE {
            // panics if csv doesn't end in newline
            remaining
                .iter()
                .enumerate()
                .filter(|(_, &el)| el == delimiter)
                .enumerate()
                .inspect(|(count, (i, _))| result[*count] = start + i)
                .count()
        } else {
            // assumes that the delimiter can be found in the next CHUNK_SIZE bytes
            // this is ok since keys are 4-22 bytes long
            let mut mask = u8x64::from_slice(&remaining[..CHUNK_SIZE]).simd_eq(splat);
            SimdCsvReader::get_indices(&mut mask, start, result)
        }
    }

    #[inline(always)]
    fn process_chunk(&mut self) -> bool {
        if self.current_pos >= self.data.len() {
            return false;
        }

        let commas_len = SimdCsvReader::find_next_delimiter(
            self.data,
            self.current_pos,
            self.simd_comma,
            b',',
            &mut self.idx_comma,
        );
        let newlines_len = SimdCsvReader::find_next_delimiter(
            self.data,
            self.current_pos,
            self.simd_newline,
            b'\n',
            &mut self.idx_newline,
        );

        for i in 0..min(commas_len, newlines_len) {
            self.queue
                .push_back(&self.data[self.current_pos..self.idx_comma[i]]);
            self.queue
                .push_back(&self.data[self.idx_comma[i] + 1..self.idx_newline[i]]);
            self.current_pos = self.idx_newline[i] + 1;
        }
        true
    }

    #[inline(always)]
    fn fill_queue(&mut self) {
        while self.queue.len() < QUEUE_CAPACITY && self.process_chunk() {}
    }
}

impl<'a> Iterator for SimdCsvReader<'a> {
    type Item = (CsvField<'a>, CsvField<'a>);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            self.fill_queue();
        }
        // there are always two
        self.queue
            .pop_front()
            .map(|e| (e, self.queue.pop_front().unwrap()))
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
        let data = b"field1,value1\nfield2,value2\nfield3,value3\n";
        let reader = data.as_slice().parse_csv();
        let results: Vec<_> = reader.collect();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], (b"field1".as_slice(), b"value1".as_slice()));
        assert_eq!(results[1], (b"field2".as_slice(), b"value2".as_slice()));
        assert_eq!(results[2], (b"field3".as_slice(), b"value3".as_slice()));
    }
}
