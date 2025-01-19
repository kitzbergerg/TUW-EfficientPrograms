const SEED: u64 = 0x517cc1b727220a95;

#[inline(always)]
pub fn compute_hash(bytes: &[u8]) -> u64 {
    // keys are 7-22 bytes long
    let r = if bytes.len() > 16 {
        let s0 = u128::from_ne_bytes(bytes[..16].try_into().unwrap());
        let s1 = u128::from_ne_bytes(bytes[bytes.len() - 16..].try_into().unwrap());
        let r = s0 * s1;
        (r as u64) ^ ((r >> 64) as u64)
    } else if bytes.len() > 8 {
        let s0 = u64::from_ne_bytes(bytes[..8].try_into().unwrap());
        let s1 = u64::from_ne_bytes(bytes[bytes.len() - 8..].try_into().unwrap());
        s0 ^ s1
    } else if bytes.len() > 4 {
        let s0 = u32::from_ne_bytes(bytes[..4].try_into().unwrap()) as u64;
        let s1 = u32::from_ne_bytes(bytes[bytes.len() - 4..].try_into().unwrap()) as u64;
        (s0 << 32) | s1
    } else {
        let s0 = bytes[0] as u64;
        let s1 = bytes[bytes.len() / 2] as u64;
        let s2 = bytes[bytes.len() - 1] as u64;
        (s0 << 16) | (s1 << 8) | s2
    };
    r.wrapping_mul(SEED)
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
