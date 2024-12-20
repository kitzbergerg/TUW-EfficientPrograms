
// src/hash.rs
include!("codegen.rs");

#[inline(always)]
pub fn compute_hash(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut hash = 0u64;
    
    // Process two bytes at a time
    unsafe {
        let ptr = bytes.as_ptr();
        let chunks = len / 2;
        
        for i in 0..chunks {
            let chunk_ptr = ptr.add(i * 2);
            let two_bytes = u16::from_ne_bytes(*chunk_ptr.cast::<[u8; 2]>());
            let value = *TWO_BYTE_VALUES.get_unchecked(two_bytes as usize) as u64;
            hash += value * *CHUNK_MULTIPLIERS.get_unchecked(i);
        }
        
        // Handle remaining byte if length is odd
        if len % 2 != 0 {
            let last_byte = *ptr.add(len - 1);
            let char_index = *CHAR_TO_INDEX.get_unchecked(last_byte as usize) as u64;
            hash += char_index * *SINGLE_MULTIPLIERS.get_unchecked(0);
        }
    }
    
    hash
}

pub struct CompileTimeHasher {
    state: u64,
}

impl std::hash::Hasher for CompileTimeHasher {
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
pub struct CompileTimeHasherBuilder;

impl std::hash::BuildHasher for CompileTimeHasherBuilder {
    type Hasher = CompileTimeHasher;

    #[inline(always)]
    fn build_hasher(&self) -> CompileTimeHasher {
        CompileTimeHasher { state: 0 }
    }
}

pub type PrecomputedHashMap<K, V> = std::collections::HashMap<K, V, CompileTimeHasherBuilder>;

#[inline(always)]
pub fn new_precomputed_hashmap<K, V>(capacity: usize) -> PrecomputedHashMap<K, V> {
    PrecomputedHashMap::with_capacity_and_hasher(capacity, CompileTimeHasherBuilder)
}
