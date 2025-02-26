use std::ops::BitXor;

const SEED: u64 = 0x517cc1b727220a95;

pub struct MyHasher {
    state: u64,
}

impl std::hash::Hasher for MyHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.state
    }

    #[inline(always)]
    fn write(&mut self, _: &[u8]) {
        unimplemented!();
    }

    fn write_u128(&mut self, i: u128) {
        let first = i as u64;
        let last = (i >> 64) as u64;
        self.state = first.wrapping_mul(SEED).bitxor(last).wrapping_mul(SEED);
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
