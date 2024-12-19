use std::{
    collections::VecDeque,
    simd::{cmp::SimdPartialEq, u8x64, LaneCount, Mask, Simd, SupportedLaneCount},
};

use smallvec::SmallVec;

use crate::CsvField;

const CHUNK_SIZE: usize = 64;
const QUEUE_CAPACITY: usize = 64;

pub struct SimdCsvReader<'a> {
    queue: VecDeque<CsvField<'a>>,
    data: &'a [u8],
    current_pos: usize,

    simd_newline: Simd<u8, CHUNK_SIZE>,
    simd_comma: Simd<u8, CHUNK_SIZE>,
}

impl<'a> SimdCsvReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        SimdCsvReader {
            queue: VecDeque::with_capacity(QUEUE_CAPACITY),
            data,
            current_pos: 0,
            simd_newline: Simd::splat(b'\n'),
            simd_comma: Simd::splat(b','),
        }
    }

    #[inline(always)]
    fn find_next_delimiter(
        &self,
        start: usize,
        splat: Simd<u8, CHUNK_SIZE>,
        delimiter: u8,
    ) -> SmallVec<[usize; 4]> {
        let remaining = &self.data[start..];
        if remaining.len() < CHUNK_SIZE {
            // panics if csv doesn't end in newline
            remaining
                .iter()
                .enumerate()
                .filter(|(_, &el)| el == delimiter)
                .map(|(i, _)| start + i)
                .collect()
        } else {
            // assumes that the delimiter can be found in the next CHUNK_SIZE bytes
            // this is ok since keys are 4-22 bytes long
            let mut mask = u8x64::from_slice(&remaining[..CHUNK_SIZE]).simd_eq(splat);
            get_indices(&mut mask, start)
        }
    }

    #[inline(always)]
    fn process_chunk(&mut self) -> bool {
        if self.current_pos >= self.data.len() {
            return false;
        }

        let commas = self.find_next_delimiter(self.current_pos, self.simd_comma, b',');
        let mut commas_iter = commas.iter();
        let newlines = self.find_next_delimiter(self.current_pos, self.simd_newline, b'\n');
        let mut newlines_iter = newlines.iter();

        while let (Some(&p1), Some(&p2)) = (commas_iter.next(), newlines_iter.next()) {
            self.queue.push_back(&self.data[self.current_pos..p1]);
            self.queue.push_back(&self.data[p1 + 1..p2]);
            self.current_pos = p2 + 1;
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

fn get_indices<const N: usize>(mask: &mut Mask<i8, N>, start: usize) -> SmallVec<[usize; 4]>
where
    LaneCount<N>: SupportedLaneCount,
{
    let mut vec = SmallVec::new_const();

    let mut idx = mask.to_bitmask().trailing_zeros() as usize;
    while idx != 64 {
        vec.push(start + idx);
        mask.set(idx, false);
        idx = mask.to_bitmask().trailing_zeros() as usize;
    }
    vec
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
    fn test_simd() {
        let data = b"field1,value1\nfield2,value2\nfield3,value3\nfield4,value4\nfield5,\n\n";

        let mut mask = u8x64::from_slice(&data[..64]).simd_eq(Simd::splat(b'\n'));
        println!("{:?}", get_indices(&mut mask, 0));
    }
}
