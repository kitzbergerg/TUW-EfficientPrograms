use std::{
    collections::VecDeque,
    simd::{cmp::SimdPartialEq, u8x64, Simd},
};

use crate::CsvField;

const CHUNK_SIZE: usize = 64;
const QUEUE_CAPACITY: usize = 64;

pub struct SimdCsvReader<'a> {
    queue: VecDeque<(CsvField<'a>, CsvField<'a>)>,
    data: &'a [u8],
    current_pos: usize,
}

impl<'a> SimdCsvReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        SimdCsvReader {
            queue: VecDeque::with_capacity(QUEUE_CAPACITY),
            data,
            current_pos: 0,
        }
    }

    #[inline(always)]
    fn find_next_delimiter(&self, start: usize, delimiter: u8) -> Option<usize> {
        let remaining = &self.data[start..];
        let chunks = remaining.chunks_exact(CHUNK_SIZE);
        let remainder = chunks.remainder();

        for (i, chunk) in chunks.enumerate() {
            let v = u8x64::from_slice(chunk);
            let mask = v.simd_eq(Simd::splat(delimiter));
            if !mask.all() {
                let idx = mask.to_bitmask().trailing_zeros();
                return Some(start + i * CHUNK_SIZE + idx as usize);
            }
        }

        remainder
            .iter()
            .position(|&x| x == delimiter)
            .map(|pos| start + remaining.len() - remainder.len() + pos)
    }

    #[inline(always)]
    fn process_chunk(&mut self) -> bool {
        let chunk_start = self.current_pos;
        let Some(newline_pos) = self.find_next_delimiter(chunk_start, b'\n') else {
            return false;
        };

        let line = &self.data[chunk_start..newline_pos];
        if line.is_empty() {
            self.current_pos = newline_pos + 1;
            return true;
        }

        if let Some(comma_pos) = line.iter().position(|&b| b == b',') {
            let field1 = &line[..comma_pos];
            let field2 = &line[comma_pos + 1..];
            self.queue.push_back((field1, field2));
        }

        self.current_pos = newline_pos + 1;
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
        self.queue.pop_front()
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

    #[test]
    fn test_empty_lines() {
        let data = b"field1,value1\n\nfield2,value2\n";
        let reader = data.as_slice().parse_csv();
        let results: Vec<_> = reader.collect();

        assert_eq!(results.len(), 2);
    }
}
