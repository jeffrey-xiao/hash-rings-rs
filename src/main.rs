extern crate hash_rings;
extern crate rand;

use hash_rings::{
    carp,
    consistent,
    jump,
    maglev,
    mpc,
    rendezvous,
    weighted_rendezvous,
};
use rand::{Rng, XorShiftRng};
use std::collections::HashMap;

const HASH_COUNT: u64 = 21;
const REPLICAS: u64 = 1611;
const ITEMS: u64 = 100_000;
const NODES: u64 = 10;

fn print_statistic(id: u64, expected: f64, actual: f64) {
    let error = (expected - actual) / actual;
    println!(
        "{:020} - Expected: {:.6} | Actual: {:.6} | Error: {:9.6}",
        id,
        expected,
        actual,
        error,
    );
}

fn test_carp_distribution() {
    println!("\nTesting carp distribution");
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let mut nodes = Vec::new();
    let mut total_weight = 0f64;

    for _ in 0..NODES {
        let id = rng.next_u64();
        let weight = rng.next_f64();

        total_weight += weight;
        occ_map.insert(id, 0f64);
        nodes.push((id, weight));
    }

    let ring = carp::Ring::new(
        nodes
            .iter()
            .map(|ref node| carp::Node::new(&node.0, node.1))
            .collect()
    );

    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1.0;
    }

    for node in &nodes {
        print_statistic(
            node.0,
            node.1 / total_weight,
            occ_map[&node.0] / ITEMS as f64,
        );
    }
}

fn test_consistent_distribution() {
    println!("\nTesting consistent distribution");
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let mut nodes = Vec::new();
    let mut ring = consistent::Ring::new();
    let total_replicas = REPLICAS * NODES;

    for _ in 0..NODES {
        let id = rng.next_u64();
        occ_map.insert(id, 0f64);
        nodes.push(id);
    }

    for node in &nodes {
        ring.insert_node(node, REPLICAS as usize);
    }

    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1.0;
    }

    for node in &nodes {
        print_statistic(
            *node,
            REPLICAS as f64 / total_replicas as f64,
            occ_map[&node] / ITEMS as f64,
        );
    }
}

fn test_jump_distribution() {
    println!("\nTesting jump distribution");
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let ring = jump::Ring::new(NODES as u32);

    for i in 0..NODES as u32 {
        occ_map.insert(i, 0f64);
    }

    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(&id).unwrap() += 1.0;
    }

    for i in 0..NODES as u32 {
        print_statistic(
            u64::from(i),
            1.0 / NODES as f64,
            occ_map[&i] / ITEMS as f64,
        );
    }
}

fn test_maglev_distribution() {
    println!("\nTesting maglev distribution");
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let mut nodes = Vec::new();

    for _ in 0..NODES {
        let id = rng.next_u64();

        occ_map.insert(id, 0);
        nodes.push(id);
    }

    let ring = maglev::Ring::new(nodes.iter().collect());

    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1;
    }

    for node in &nodes {
        print_statistic(
            *node,
            1.0 / NODES as f64,
            f64::from(occ_map[&node]) / ITEMS as f64,
        );
    }
}

fn test_mpc_distribution() {
    println!("\nTesting mpc distribution");
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let mut nodes = Vec::new();
    let mut ring = mpc::Ring::new(HASH_COUNT);

    for _ in 0..NODES {
        let id = rng.next_u64();
        occ_map.insert(id, 0);
        nodes.push(id);
    }

    for node in &nodes {
        ring.insert_node(node);
    }

    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1;
    }

    for node in &nodes {
        print_statistic(
            *node,
            1.0 / NODES as f64,
            f64::from(occ_map[&node]) / ITEMS as f64,
        );
    }
}

fn test_rendezvous_distribution() {
    println!("\nTesting rendezvous distribution");
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let mut nodes = Vec::new();
    let mut ring = rendezvous::Ring::new();

    for _ in 0..NODES {
        let id = rng.next_u64();
        occ_map.insert(id, 0f64);
        nodes.push(id);
    }

    for node in &nodes {
        ring.insert_node(node, 1);
    }

    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1.0;
    }

    for node in &nodes {
        print_statistic(
            *node,
            1.0 / NODES as f64,
            occ_map[&node] / ITEMS as f64,
        );
    }
}

fn test_weighted_rendezvous_distribution() {
    println!("\nTesting weighted rendezvous distribution");
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let mut nodes = Vec::new();
    let mut ring = weighted_rendezvous::Ring::new();
    let mut total_weight = 0f64;

    for _ in 0..NODES {
        let id = rng.next_u64();
        let weight = rng.next_f64();

        total_weight += weight;
        occ_map.insert(id, 0f64);
        nodes.push((id, weight));
    }

    for node in &nodes {
        ring.insert_node(&node.0, node.1);
    }

    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1.0;
    }

    for node in &nodes {
        print_statistic(
            node.0,
            node.1 as f64 / total_weight as f64,
            occ_map[&node.0] / ITEMS as f64,
        );
    }
}

fn main() {
    test_carp_distribution();
    test_consistent_distribution();
    test_jump_distribution();
    test_maglev_distribution();
    test_mpc_distribution();
    test_rendezvous_distribution();
    test_weighted_rendezvous_distribution();
}
