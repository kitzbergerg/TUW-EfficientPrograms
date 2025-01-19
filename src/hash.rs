use bytemuck::cast;
use std::ops::Mul;

const SEED: u64 = 0x517cc1b727220a95;

macro_rules! hash_simd {
    ($bytes:expr, $size:expr) => {{
        let start = std::simd::Simd::<u8, $size>::from_array($bytes[..$size].try_into().unwrap());
        let end = std::simd::Simd::<u8, $size>::from_array(
            $bytes[$bytes.len() - $size..].try_into().unwrap(),
        );

        // Mix using SIMD operations
        let mixed = start
            .rotate_elements_left::<3>()
            .mul(end)
            .rotate_elements_right::<5>();

        cast(*mixed.as_array())
    }};
}

#[inline(always)]
pub fn compute_hash(bytes: &[u8]) -> u64 {
    // keys are 7-22 bytes long
    if bytes.len() > 16 {
        let result: [u64; 2] = hash_simd!(bytes, 16);
        (result[0] ^ result[1]) * SEED
    } else if bytes.len() > 8 {
        let result: u64 = hash_simd!(bytes, 8);
        result * SEED
    } else {
        bytes.iter().fold(0, |acc, &b| (acc ^ (b as u64)) * SEED)
    }
}

pub struct MyHasher {
    state: u64,
}

impl std::hash::Hasher for MyHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.state
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        self.state = compute_hash(bytes);
    }
}

#[derive(Default)]
pub struct MyHasherBuilder;

impl std::hash::BuildHasher for MyHasherBuilder {
    type Hasher = MyHasher;

    #[inline(always)]
    fn build_hasher(&self) -> MyHasher {
        MyHasher { state: 0 }
    }
}

pub type MyHashMap<K, V> = std::collections::HashMap<K, V, MyHasherBuilder>;
