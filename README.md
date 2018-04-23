# hash-rings-rs
[![hash-rings](http://meritbadge.herokuapp.com/hash-rings)](https://crates.io/crates/hash-rings)
[![Documentation](https://docs.rs/hash-rings/badge.svg)](https://docs.rs/hash-rings)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://travis-ci.org/jeffrey-xiao/hash-rings-rs.svg?branch=master)](https://travis-ci.org/jeffrey-xiao/hash-rings-rs)
[![codecov](https://codecov.io/gh/jeffrey-xiao/hash-rings-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/jeffrey-xiao/hash-rings-rs)

`hash-rings` contains implementations for seven different hash ring algorithms: Cache Array Routing Protocol, Consistent Hashing, Multi-Probe Consistent Hashing, Rendezvous Hashing, Weighted Rendezvous Hashing, Maglev Hashing, and Jump Hashing. It also provides clients for Consistent Hashing, Rendezvous Hashing, and Weighted Rendezvous Hashing to efficiently redistribute items as nodes are inserted and removed from the ring.

## Examples
### Example Ring Usage
```rust
extern crate hash_rings;

use hash_rings::consistent::Ring;

fn main() {
    let mut r = Ring::new();

    r.insert_node(&"node-1", 1);
    r.insert_node(&"node-2", 3);

    assert_eq!(r.get_node(&"point-1"), &"node-1");
}
```

### Example Client Usage
```rust
extern crate hash_rings;

use hash_rings::consistent::Client;

fn main() {
    let mut c = Client::new();
    c.insert_node(&"node-1", 1);
    c.insert_node(&"node-2", 3);

    c.insert_point(&"point-1");

    assert_eq!(c.get_node(&"point-1"), &"node-1");
    assert_eq!(c.get_points(&"node-1"), [&"point-1"]);
}
```

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
hash-rings = "*"
```
and this to your crate root:
```rust
extern crate hash_rings;
```

## References
 - [Cache Array Routing Protocol](https://tools.ietf.org/html/draft-vinod-carp-v1-03)
 - [New Hashing Algorithms for Data Storage](http://www.snia.org/sites/default/files/SDC15_presentations/dist_sys/Jason_Resch_New_Consistent_Hashings_Rev.pdf)
 - [A Fast, Minimal Memory, Consistent Hash Algorithm](https://arxiv.org/abs/1406.2294)
 - [Maglev: A Fast and Reliable Software Network Load Balancer](https://research.google.com/pubs/pub44824.html)
 - [Multi-probe consistent hashing](https://arxiv.org/abs/1505.00062)
 - [Consistent hashing and random trees: distributed caching protocols for relieving hot spots on the World Wide Web](https://dl.acm.org/citation.cfm?id=258660)
 > David Karger, Eric Lehman, Tom Leighton, Rina Panigrahy, Matthew Levine, and Daniel Lewin. 1997. Consistent hashing and random trees: distributed caching protocols for relieving hot spots on the World Wide Web. In Proceedings of the twenty-ninth annual ACM symposium on Theory of computing (STOC '97). ACM, New York, NY, USA, 654-663. DOI=http://dx.doi.org/10.1145/258533.258660
 - [Using name-based mappings to increase hit rates](https://dl.acm.org/citation.cfm?id=276288)
 > David G. Thaler and Chinya V. Ravishankar. 1998. Using name-based mappings to increase hit rates. IEEE/ACM Trans. Netw. 6, 1 (February 1998), 1-14. DOI=http://dx.doi.org/10.1109/90.663936
