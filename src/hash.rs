use std::ops::Mul;

// Carefully chosen prime multipliers for good distribution
const MULT_A: u64 = 0x517cc1b727220a95;
const MULT_B: u64 = 0x9e3779b97f4a7c15;

macro_rules! hash_simd {
    ($bytes:expr, $size:expr, $t:ty) => {{
        // SAFETY: If the slice is at least as long as $size bytes it can be reinterpreted as an actual array.
        let start: [u8; $size] = *$bytes.as_ptr().cast();
        let end: [u8; $size] = *$bytes.as_ptr().add($bytes.len() - $size).cast();

        let start = std::simd::Simd::<u8, $size>::from_array(start);
        let end = std::simd::Simd::<u8, $size>::from_array(end);

        // Mix using SIMD operations
        let mixed = start
            .rotate_elements_left::<3>()
            .mul(end)
            .rotate_elements_right::<5>();

        // SAFETY: assert at compile time that input and output have same size
        const _: () = assert!(size_of::<$t>() == size_of::<[u8; $size]>());
        *(mixed.as_array().as_ptr().cast::<$t>())
    }};
}

#[inline(always)]
pub fn compute_hash(bytes: &[u8]) -> u64 {
    // keys are 7-22 bytes long
    if bytes.len() > 16 {
        let result = unsafe { hash_simd!(bytes, 16, [u64; 2]) };
        result[0].wrapping_mul(MULT_A) ^ result[1].wrapping_mul(MULT_B)
    } else if bytes.len() > 8 {
        let result = unsafe { hash_simd!(bytes, 8, u64) };
        result.wrapping_mul(MULT_A)
    } else {
        let result = unsafe { hash_simd!(bytes, 4, u32) };
        (result as u64).wrapping_mul(MULT_A)
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
