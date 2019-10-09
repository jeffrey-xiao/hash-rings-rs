use std::collections::hash_map::DefaultHasher;
use std::hash::{BuildHasherDefault, Hasher};

pub struct AddHasher {
    sum: u64,
}

impl Hasher for AddHasher {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.sum += u64::from(*byte);
        }
    }

    fn finish(&self) -> u64 {
        self.sum
    }
}

impl Default for AddHasher {
    fn default() -> Self {
        Self { sum: 0 }
    }
}

pub type BuildAddHasher = BuildHasherDefault<AddHasher>;
pub type BuildDefaultHasher = BuildHasherDefault<DefaultHasher>;
