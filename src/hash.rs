use std::ops::BitXor;

const SEED: u64 = 0x517cc1b727220a95;

#[inline(always)]
pub fn compute_hash(bytes: &[u8]) -> u64 {
    // keys are 7-22 bytes long
    if bytes.len() > 16 {
        let start = u64::from_ne_bytes(bytes[..8].try_into().unwrap());
        let mid = u64::from_ne_bytes(bytes[8..16].try_into().unwrap());
        let end = u64::from_ne_bytes(bytes[bytes.len() - 8..].try_into().unwrap());

        start
            .wrapping_mul(SEED)
            .bitxor(mid)
            .wrapping_mul(SEED)
            .bitxor(end)
            .wrapping_mul(SEED)
    } else if bytes.len() > 8 {
        let start = u64::from_ne_bytes(bytes[..8].try_into().unwrap());
        let end = u64::from_ne_bytes(bytes[bytes.len() - 8..].try_into().unwrap());

        start.wrapping_mul(SEED).bitxor(end).wrapping_mul(SEED)
    } else {
        let start = u32::from_ne_bytes(bytes[..4].try_into().unwrap()) as u64;
        let end = u32::from_ne_bytes(bytes[bytes.len() - 4..].try_into().unwrap()) as u64;

        start.wrapping_mul(SEED).bitxor(end).wrapping_mul(SEED)
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
