extern crate hash_rings;

use hash_rings::consistent::Client;

fn main() {
    let mut c = Client::new();
    c.insert_node(&"node-1", 1);
    c.insert_node(&"node-2", 3);

    c.insert_point(&"point-1");

    assert_eq!(c.get_node(&"point-1"), &"node-2");
    assert_eq!(c.get_points(&"node-2"), [&"point-1"]);
}
