use hash_rings::consistent::Ring;

fn main() {
    let mut r = Ring::new();

    r.insert_node(&"node-1", 1);
    r.insert_node(&"node-2", 3);

    assert_eq!(r.get_node(&"point-1"), &"node-2");
}
