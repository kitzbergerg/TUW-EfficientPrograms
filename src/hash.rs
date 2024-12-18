use std::{collections::HashMap, hash::BuildHasherDefault};
#[derive(Default)]
pub struct MyHasher {
    state: u64,
}

fn custom_hash(key: &[u8]) -> u64 {
    let mut hash = 0u64;

    for ch in key.iter() {
        let val = match ch {
            b'A'..=b'Z' => *ch as u64 - b'A' as u64,      // A-Z -> 0-25
            b'0'..=b'9' => *ch as u64 - b'0' as u64 + 26, // 0-9 -> 26-35
            _ => continue,      // Ignore control chars
        };

        // Incorporate the character into the hash, with a bit shift to spread bits.
        hash = hash.wrapping_mul(31).wrapping_add(val);

        // Mix the hash further with bit shifts to ensure uniform distribution.
        hash = hash ^ (hash >> 33);
        hash = hash.wrapping_add(hash << 21);
        hash = hash ^ (hash >> 56);
    }

    // Return the final 64-bit hash value
    hash
}

impl std::hash::Hasher for MyHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.state = custom_hash(bytes)
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

#[derive(Default)]
pub struct BuildMyHasher;

impl std::hash::BuildHasher for BuildMyHasher {
    type Hasher = MyHasher;

    fn build_hasher(&self) -> MyHasher {
        MyHasher { state: 0 }
    }
}

pub type MyBuildHasher = BuildHasherDefault<MyHasher>;
pub type MyHashMap<K, V> = HashMap<K, V, MyBuildHasher>;
