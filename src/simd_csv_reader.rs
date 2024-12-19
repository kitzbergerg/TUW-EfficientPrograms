use std::{
    collections::VecDeque,
    simd::{cmp::SimdPartialEq, u8x32, Simd},
};

use crate::CsvField;

const CHUNK_SIZE: usize = 32;
const QUEUE_CAPACITY: usize = 64;

pub struct SimdCsvReader<'a> {
    queue: VecDeque<CsvField<'a>>,
    data: &'a [u8],
    current_pos: usize,

    simd_newline: Simd<u8, CHUNK_SIZE>,
    simd_comma: Simd<u8, CHUNK_SIZE>,
    next_is_newline: bool,
}

impl<'a> SimdCsvReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        SimdCsvReader {
            queue: VecDeque::with_capacity(QUEUE_CAPACITY),
            data,
            current_pos: 0,
            simd_newline: Simd::splat(b'\n'),
            simd_comma: Simd::splat(b','),
            next_is_newline: false,
        }
    }

    #[inline(always)]
    fn find_next_delimiter(
        &self,
        start: usize,
        splat: Simd<u8, CHUNK_SIZE>,
        delimiter: u8,
    ) -> usize {
        let remaining = &self.data[start..];
        if remaining.len() < CHUNK_SIZE {
            // panics if csv doesn't end in newline
            remaining
                .iter()
                .position(|&x| x == delimiter)
                .map(|pos| start + remaining.len() - remaining.len() + pos)
                .unwrap()
        } else {
            // assumes that the delimiter can be found in the next CHUNK_SIZE bytes
            // this is ok since keys are 4-22 bytes long
            let mask = u8x32::from_slice(&remaining[..CHUNK_SIZE]).simd_eq(splat);
            let idx = mask.to_bitmask().trailing_zeros();
            start + idx as usize
        }
    }

    #[inline(always)]
    fn process_chunk(&mut self) -> bool {
        let chunk_start = self.current_pos;
        if chunk_start >= self.data.len() {
            return false;
        }

        let pos = if self.next_is_newline {
            self.find_next_delimiter(chunk_start, self.simd_newline, b'\n')
        } else {
            self.find_next_delimiter(chunk_start, self.simd_comma, b',')
        };

        let field = &self.data[chunk_start..pos];
        self.queue.push_back(field);
        self.current_pos = pos + 1;
        self.next_is_newline = !self.next_is_newline;
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
