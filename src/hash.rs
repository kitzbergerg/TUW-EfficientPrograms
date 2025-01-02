use std::{
    ops::Mul,
    simd::{u8x4, u8x8, u8x16},
};

use bytemuck::cast;

// Carefully chosen prime multipliers for good distribution
const MULT_A: u64 = 0x517cc1b727220a95;
const MULT_B: u64 = 0x9e3779b97f4a7c15;

#[inline(always)]
pub fn compute_hash(bytes: &[u8]) -> u64 {
    // keys are 7-22 bytes long
    if bytes.len() > 16 {
        let start = u8x16::from_array(bytes[..16].try_into().unwrap());
        let end = u8x16::from_array(bytes[bytes.len() - 16..].try_into().unwrap());
        // Mix using SIMD operations
        let mixed = start
            .rotate_elements_left::<3>()
            .mul(end)
            .rotate_elements_right::<5>();

        // Map to u64
        let cast: [u64; 2] = cast(*mixed.as_array());
        cast[0].wrapping_mul(MULT_A) ^ cast[1].wrapping_mul(MULT_B)
    } else if bytes.len() > 8 {
        let start = u8x8::from_array(bytes[..8].try_into().unwrap());
        let end = u8x8::from_array(bytes[bytes.len() - 8..].try_into().unwrap());
        // Mix using SIMD operations
        let mixed = start
            .rotate_elements_left::<3>()
            .mul(end)
            .rotate_elements_right::<5>();

        // Map to u64
        let cast: u64 = cast(*mixed.as_array());
        cast.wrapping_mul(MULT_A)
    } else {
        let start = u8x4::from_array(bytes[..4].try_into().unwrap());
        let end = u8x4::from_array(bytes[bytes.len() - 4..].try_into().unwrap());
        // Mix using SIMD operations
        let mixed = start
            .rotate_elements_left::<3>()
            .mul(end)
            .rotate_elements_right::<5>();

        // Map to u64
        let cast: u32 = cast(*mixed.as_array());
        (cast as u64).wrapping_mul(MULT_A)
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
