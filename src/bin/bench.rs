extern crate hash_rings;
extern crate rand;

use hash_rings::{carp, consistent, jump, maglev, mpc, rendezvous, weighted_rendezvous};
use rand::{Rng, XorShiftRng};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const HASH_COUNT: u64 = 21;
const REPLICAS: u64 = 1611;
const ITEMS: u64 = 100_000;
const NODES: u64 = 10;

fn print_node_statistic(id: u64, expected: f64, actual: f64) {
    let error = (expected - actual) / actual;
    println!(
        "{:020} - Expected: {:.6} | Actual: {:.6} | Error: {:9.6}",
        id, expected, actual, error,
    );
}

fn print_bench_statistic(duration: Duration) {
    let total_time = duration.as_secs() as f64 * 1e9 + f64::from(duration.subsec_nanos());
    let ns_per_op = total_time / ITEMS as f64;
    let ops_per_ns = 1e9 / ns_per_op;
    println!();
    println!("Total elapsed time:         {:>10.3} ms", total_time / 1e6);
    println!("Milliseconds per operation: {:>10.3} ns", ns_per_op);
    println!("Operations per second:      {:>10.3} op/ms", ops_per_ns);
    println!();
}

fn bench_carp() {
    println!("\nBenching carp hashing ({} nodes, {} items)", NODES, ITEMS);
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
            .map(|node| carp::Node::new(&node.0, node.1))
            .collect(),
    );

    let start = Instant::now();
    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1.0;
    }

    for node in &nodes {
        print_node_statistic(
            node.0,
            node.1 / total_weight,
            occ_map[&node.0] / ITEMS as f64,
        );
    }
    print_bench_statistic(start.elapsed());
}

fn bench_consistent() {
    println!(
        "\nBenching consistent hashing ({} nodes, {} replicas, {} items)",
        NODES, REPLICAS, ITEMS,
    );
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

    let start = Instant::now();
    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1.0;
    }

    for node in &nodes {
        print_node_statistic(
            *node,
            REPLICAS as f64 / total_replicas as f64,
            occ_map[&node] / ITEMS as f64,
        );
    }
    print_bench_statistic(start.elapsed());
}

fn bench_jump() {
    println!("\nBenching jump hashing ({} nodes, {} items)", NODES, ITEMS,);
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let ring = jump::Ring::new(NODES as u32);

    for i in 0..NODES as u32 {
        occ_map.insert(i, 0f64);
    }

    let start = Instant::now();
    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(&id).unwrap() += 1.0;
    }

    for i in 0..NODES as u32 {
        print_node_statistic(u64::from(i), 1.0 / NODES as f64, occ_map[&i] / ITEMS as f64);
    }
    print_bench_statistic(start.elapsed());
}

fn bench_maglev() {
    println!(
        "\nBenching maglev hashing ({} nodes, {} items)",
        NODES, ITEMS,
    );
    let mut rng = XorShiftRng::new_unseeded();

    let mut occ_map = HashMap::new();
    let mut nodes = Vec::new();

    for _ in 0..NODES {
        let id = rng.next_u64();

        occ_map.insert(id, 0);
        nodes.push(id);
    }

    let ring = maglev::Ring::new(nodes.iter().collect());

    let start = Instant::now();
    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1;
    }

    for node in &nodes {
        print_node_statistic(
            *node,
            1.0 / NODES as f64,
            f64::from(occ_map[&node]) / ITEMS as f64,
        );
    }
    print_bench_statistic(start.elapsed());
}

fn bench_mpc() {
    println!(
        "\nBenching mpc hashing ({} nodes, {} probes, {} items)",
        NODES, HASH_COUNT, ITEMS,
    );
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

    let start = Instant::now();
    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1;
    }

    for node in &nodes {
        print_node_statistic(
            *node,
            1.0 / NODES as f64,
            f64::from(occ_map[&node]) / ITEMS as f64,
        );
    }
    print_bench_statistic(start.elapsed());
}

fn bench_rendezvous() {
    println!(
        "\nBenching rendezvous hashing ({} nodes, {} items)",
        NODES, ITEMS,
    );
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

    let start = Instant::now();
    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1.0;
    }

    for node in &nodes {
        print_node_statistic(*node, 1.0 / NODES as f64, occ_map[&node] / ITEMS as f64);
    }
    print_bench_statistic(start.elapsed());
}

fn bench_weighted_rendezvous() {
    println!(
        "\nBenching weighted rendezvous hashing ({} nodes, {} items)",
        NODES, ITEMS,
    );
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

    let start = Instant::now();
    for _ in 0..ITEMS {
        let id = ring.get_node(&rng.next_u64());
        *occ_map.get_mut(id).unwrap() += 1.0;
    }

    for node in &nodes {
        print_node_statistic(
            node.0,
            node.1 as f64 / total_weight as f64,
            occ_map[&node.0] / ITEMS as f64,
        );
    }
    print_bench_statistic(start.elapsed());
}

fn main() {
    bench_carp();
    bench_consistent();
    bench_jump();
    bench_maglev();
    bench_mpc();
    bench_rendezvous();
    bench_weighted_rendezvous();
}
