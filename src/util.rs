use std::hash::{BuildHasher, Hash, Hasher};

pub fn gen_hash<T, H>(hash_builder: &H, value: &T) -> u64
where
    T: Hash,
    H: BuildHasher,
{
    let mut hasher = hash_builder.build_hasher();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn combine_hash<H>(hash_builder: &H, x: u64, y: u64) -> u64
where
    H: BuildHasher,
{
    let mut hasher = hash_builder.build_hasher();
    x.hash(&mut hasher);
    y.hash(&mut hasher);
    hasher.finish()
}
