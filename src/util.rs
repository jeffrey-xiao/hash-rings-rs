use siphasher::sip::SipHasher;
use std::hash::{Hash, Hasher};

pub fn gen_hash<T>(value: &T) -> u64
where T: Hash {
    let mut hasher = SipHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn combine_hash(x: u64, y: u64) -> u64 {
    let mut hasher = SipHasher::new();
    x.hash(&mut hasher);
    y.hash(&mut hasher);
    hasher.finish()
}
