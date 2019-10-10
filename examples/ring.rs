use hash_rings::consistent::Ring;
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;

fn main() {
    let mut r = Ring::with_hasher(DefaultBuildHasher::default());

    r.insert_node(&"node-1", 1);
    r.insert_node(&"node-2", 3);

    assert_eq!(r.get_node(&"point-1"), &"node-1");
}
