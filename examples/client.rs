use hash_rings::consistent::Client;
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;

fn main() {
    let mut c = Client::with_hasher(DefaultBuildHasher::default());
    c.insert_node(&"node-1", 1);
    c.insert_node(&"node-2", 3);

    c.insert_point(&"point-1");

    assert_eq!(c.get_node(&"point-1"), &"node-1");
    assert_eq!(c.get_points(&"node-1"), [&"point-1"]);
}
