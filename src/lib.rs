//! # hash-rings-rs
//!
//! [![hash-rings](http://meritbadge.herokuapp.com/hash-rings)](https://crates.io/crates/hash-rings)
//! [![Documentation](https://docs.rs/hash-rings/badge.svg)](https://docs.rs/hash-rings)
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//! [![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
//! [![Build Status](https://travis-ci.org/jeffrey-xiao/hash-rings-rs.svg?branch=master)](https://travis-ci.org/jeffrey-xiao/hash-rings-rs)
//! [![codecov](https://codecov.io/gh/jeffrey-xiao/hash-rings-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/jeffrey-xiao/hash-rings-rs)
//!
//! `hash-rings` contains implementations for seven different hash ring algorithms: Cache Array
//! Routing Protocol, Consistent Hashing, Multi-Probe Consistent Hashing, Rendezvous Hashing,
//! Weighted Rendezvous Hashing, Maglev Hashing, and Jump Hashing. It also provides clients for
//! Consistent Hashing, Rendezvous Hashing, and Weighted Rendezvous Hashing to efficiently
//! redistribute items as nodes are inserted and removed from the ring.
//!
//! ## Examples
//!
//! ### Example Ring Usage
//!
//! ```rust
//! use hash_rings::consistent::Ring;
//!
//! fn main() {
//!     let mut r = Ring::new();
//!
//!     r.insert_node(&"node-1", 1);
//!     r.insert_node(&"node-2", 3);
//!
//!     assert_eq!(r.get_node(&"point-1"), &"node-2");
//! }
//! ```
//!
//! ### Example Client Usage
//!
//! ```rust
//! use hash_rings::consistent::Client;
//!
//! fn main() {
//!     let mut c = Client::new();
//!     c.insert_node(&"node-1", 1);
//!     c.insert_node(&"node-2", 3);
//!
//!     c.insert_point(&"point-1");
//!
//!     assert_eq!(c.get_node(&"point-1"), &"node-2");
//!     assert_eq!(c.get_points(&"node-2"), [&"point-1"]);
//! }
//! ```
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hash-rings = "*"
//! ```
//!
//! and this to your crate root if you are using Rust 2015:
//!
//! ```rust
//! extern crate hash_rings;
//! ```
//!
//! ## Benchmarks
//!
//! ```text
//! Benching carp hashing (10 nodes, 100000 items)
//! 15848556381555908996 - Expected: 0.155015 | Actual: 0.155180 | Error: -0.001060
//! 06801744144136471498 - Expected: 0.056593 | Actual: 0.056960 | Error: -0.006447
//! 16730135874920933484 - Expected: 0.015944 | Actual: 0.016030 | Error: -0.005355
//! 11802923454833793349 - Expected: 0.135407 | Actual: 0.134050 | Error:  0.010122
//! 14589965171469706430 - Expected: 0.091974 | Actual: 0.093030 | Error: -0.011348
//! 06790293794189608791 - Expected: 0.122949 | Actual: 0.123230 | Error: -0.002284
//! 08283237945741952176 - Expected: 0.042317 | Actual: 0.042880 | Error: -0.013126
//! 06540337216311911463 - Expected: 0.146495 | Actual: 0.145220 | Error:  0.008782
//! 13241461372147825909 - Expected: 0.084205 | Actual: 0.084330 | Error: -0.001484
//! 06769854041949442045 - Expected: 0.149100 | Actual: 0.149090 | Error:  0.000070
//!
//! Total elapsed time:           1336.552 ms
//! Milliseconds per operation:  13365.519 ns
//! Operations per second:       74819.391 op/ms
//!
//!
//! Benching consistent hashing (10 nodes, 1611 replicas, 100000 items)
//! 15848556381555908996 - Expected: 0.100000 | Actual: 0.102070 | Error: -0.020280
//! 13987966085338848396 - Expected: 0.100000 | Actual: 0.102410 | Error: -0.023533
//! 06801744144136471498 - Expected: 0.100000 | Actual: 0.102240 | Error: -0.021909
//! 04005265977620077421 - Expected: 0.100000 | Actual: 0.100010 | Error: -0.000100
//! 16730135874920933484 - Expected: 0.100000 | Actual: 0.098970 | Error:  0.010407
//! 13195988079190323012 - Expected: 0.100000 | Actual: 0.099630 | Error:  0.003714
//! 11802923454833793349 - Expected: 0.100000 | Actual: 0.102730 | Error: -0.026575
//! 05146857450694500275 - Expected: 0.100000 | Actual: 0.099290 | Error:  0.007151
//! 14589965171469706430 - Expected: 0.100000 | Actual: 0.098170 | Error:  0.018641
//! 17291863876572781215 - Expected: 0.100000 | Actual: 0.094480 | Error:  0.058425
//!
//! Total elapsed time:            417.016 ms
//! Milliseconds per operation:   4170.163 ns
//! Operations per second:      239798.789 op/ms
//!
//!
//! Benching jump hashing (10 nodes, 100000 items)
//! 00000000000000000000 - Expected: 0.100000 | Actual: 0.098250 | Error:  0.017812
//! 00000000000000000001 - Expected: 0.100000 | Actual: 0.100140 | Error: -0.001398
//! 00000000000000000002 - Expected: 0.100000 | Actual: 0.100280 | Error: -0.002792
//! 00000000000000000003 - Expected: 0.100000 | Actual: 0.100240 | Error: -0.002394
//! 00000000000000000004 - Expected: 0.100000 | Actual: 0.101550 | Error: -0.015263
//! 00000000000000000005 - Expected: 0.100000 | Actual: 0.099290 | Error:  0.007151
//! 00000000000000000006 - Expected: 0.100000 | Actual: 0.100750 | Error: -0.007444
//! 00000000000000000007 - Expected: 0.100000 | Actual: 0.100130 | Error: -0.001298
//! 00000000000000000008 - Expected: 0.100000 | Actual: 0.098730 | Error:  0.012863
//! 00000000000000000009 - Expected: 0.100000 | Actual: 0.100640 | Error: -0.006359
//!
//! Total elapsed time:            191.231 ms
//! Milliseconds per operation:   1912.314 ns
//! Operations per second:      522926.543 op/ms
//!
//!
//! Benching maglev hashing (10 nodes, 100000 items)
//! 15848556381555908996 - Expected: 0.100000 | Actual: 0.099670 | Error:  0.003311
//! 13987966085338848396 - Expected: 0.100000 | Actual: 0.100700 | Error: -0.006951
//! 06801744144136471498 - Expected: 0.100000 | Actual: 0.099130 | Error:  0.008776
//! 04005265977620077421 - Expected: 0.100000 | Actual: 0.099960 | Error:  0.000400
//! 16730135874920933484 - Expected: 0.100000 | Actual: 0.101340 | Error: -0.013223
//! 13195988079190323012 - Expected: 0.100000 | Actual: 0.098740 | Error:  0.012761
//! 11802923454833793349 - Expected: 0.100000 | Actual: 0.100650 | Error: -0.006458
//! 05146857450694500275 - Expected: 0.100000 | Actual: 0.101050 | Error: -0.010391
//! 14589965171469706430 - Expected: 0.100000 | Actual: 0.100660 | Error: -0.006557
//! 17291863876572781215 - Expected: 0.100000 | Actual: 0.098100 | Error:  0.019368
//!
//! Total elapsed time:            188.203 ms
//! Milliseconds per operation:   1882.027 ns
//! Operations per second:      531342.016 op/ms
//!
//!
//! Benching mpc hashing (10 nodes, 21 probes, 100000 items)
//! 15848556381555908996 - Expected: 0.100000 | Actual: 0.096820 | Error:  0.032844
//! 13987966085338848396 - Expected: 0.100000 | Actual: 0.098510 | Error:  0.015125
//! 06801744144136471498 - Expected: 0.100000 | Actual: 0.103730 | Error: -0.035959
//! 04005265977620077421 - Expected: 0.100000 | Actual: 0.093530 | Error:  0.069176
//! 16730135874920933484 - Expected: 0.100000 | Actual: 0.103210 | Error: -0.031102
//! 13195988079190323012 - Expected: 0.100000 | Actual: 0.083890 | Error:  0.192037
//! 11802923454833793349 - Expected: 0.100000 | Actual: 0.096990 | Error:  0.031034
//! 05146857450694500275 - Expected: 0.100000 | Actual: 0.111780 | Error: -0.105386
//! 14589965171469706430 - Expected: 0.100000 | Actual: 0.098680 | Error:  0.013377
//! 17291863876572781215 - Expected: 0.100000 | Actual: 0.112860 | Error: -0.113946
//!
//! Total elapsed time:           1153.555 ms
//! Milliseconds per operation:  11535.552 ns
//! Operations per second:       86688.529 op/ms
//!
//!
//! Benching rendezvous hashing (10 nodes, 100000 items)
//! 15848556381555908996 - Expected: 0.100000 | Actual: 0.099680 | Error:  0.003210
//! 13987966085338848396 - Expected: 0.100000 | Actual: 0.100710 | Error: -0.007050
//! 06801744144136471498 - Expected: 0.100000 | Actual: 0.100320 | Error: -0.003190
//! 04005265977620077421 - Expected: 0.100000 | Actual: 0.099820 | Error:  0.001803
//! 16730135874920933484 - Expected: 0.100000 | Actual: 0.099900 | Error:  0.001001
//! 13195988079190323012 - Expected: 0.100000 | Actual: 0.100600 | Error: -0.005964
//! 11802923454833793349 - Expected: 0.100000 | Actual: 0.098440 | Error:  0.015847
//! 05146857450694500275 - Expected: 0.100000 | Actual: 0.099440 | Error:  0.005632
//! 14589965171469706430 - Expected: 0.100000 | Actual: 0.101420 | Error: -0.014001
//! 17291863876572781215 - Expected: 0.100000 | Actual: 0.099670 | Error:  0.003311
//!
//! Total elapsed time:           1623.272 ms
//! Milliseconds per operation:  16232.719 ns
//! Operations per second:       61603.972 op/ms
//!
//!
//! Benching weighted rendezvous hashing (10 nodes, 100000 items)
//! 15848556381555908996 - Expected: 0.155015 | Actual: 0.154470 | Error:  0.003531
//! 06801744144136471498 - Expected: 0.056593 | Actual: 0.057320 | Error: -0.012687
//! 16730135874920933484 - Expected: 0.015944 | Actual: 0.016210 | Error: -0.016400
//! 11802923454833793349 - Expected: 0.135407 | Actual: 0.134700 | Error:  0.005248
//! 14589965171469706430 - Expected: 0.091974 | Actual: 0.092940 | Error: -0.010391
//! 06790293794189608791 - Expected: 0.122949 | Actual: 0.123490 | Error: -0.004385
//! 08283237945741952176 - Expected: 0.042317 | Actual: 0.042200 | Error:  0.002776
//! 06540337216311911463 - Expected: 0.146495 | Actual: 0.144770 | Error:  0.011918
//! 13241461372147825909 - Expected: 0.084205 | Actual: 0.083530 | Error:  0.008080
//! 06769854041949442045 - Expected: 0.149100 | Actual: 0.150370 | Error: -0.008443
//!
//! Total elapsed time:           2233.020 ms
//! Milliseconds per operation:  22330.205 ns
//! Operations per second:       44782.393 op/ms
//! ```
//!
//! ## Changelog
//!
//! See [CHANGELOG](CHANGELOG.md) for more details.
//!
//! ## References
//!
//! - [A Fast, Minimal Memory, Consistent Hash Algorithm](https://arxiv.org/abs/1406.2294)
//!   > Lamping, John, and Eric Veach. 2014. “A Fast, Minimal Memory, Consistent Hash Algorithm.” *CoRR* abs/1406.2294. <http://arxiv.org/abs/1406.2294>.
//! - [Cache Array Routing Protocol](https://tools.ietf.org/html/draft-vinod-carp-v1-03)
//! - [Consistent hashing and random trees: distributed caching protocols for relieving hot spots on the World Wide Web](https://dl.acm.org/citation.cfm?id=258660)
//!   > Karger, David, Eric Lehman, Tom Leighton, Rina Panigrahy, Matthew Levine, and Daniel Lewin. 1997. “Consistent Hashing and Random Trees: Distributed Caching Protocols for Relieving Hot Spots on the World Wide Web.” In *Proceedings of the Twenty-Ninth Annual Acm Symposium on Theory of Computing*, 654–63. STOC ’97. New York, NY, USA: ACM. doi:[10.1145/258533.258660](https://doi.org/10.1145/258533.258660).
//! - [Maglev: A Fast and Reliable Software Network Load Balancer](https://research.google.com/pubs/pub44824.html)
//!   > Eisenbud, Daniel E., Cheng Yi, Carlo Contavalli, Cody Smith, Roman Kononov, Eric Mann-Hielscher, Ardas Cilingiroglu, Bin Cheyney, Wentao Shang, and Jinnah Dylan Hosein. 2016. “Maglev: A Fast and Reliable Software Network Load Balancer.” In *13th Usenix Symposium on Networked Systems Design and Implementation (Nsdi 16)*, 523–35. Santa Clara, CA. <https://www.usenix.org/conference/nsdi16/technical-sessions/presentation/eisenbud>.
//! - [Multi-probe consistent hashing](https://arxiv.org/abs/1505.00062)
//!   > Appleton, Ben, and Michael O’Reilly. 2015. “Multi-Probe Consistent Hashing.” *CoRR* abs/1505.00062. <http://arxiv.org/abs/1505.00062>.
//! - [Using name-based mappings to increase hit rates](https://dl.acm.org/citation.cfm?id=276288)
//!   > Thaler, David G., and Chinya V. Ravishankar. 1998. “Using Name-Based Mappings to Increase Hit Rates.” *IEEE/ACM Trans. Netw.* 6 (1). Piscataway, NJ, USA: IEEE Press: 1–14. doi:[10.1109/90.663936](https://doi.org/10.1109/90.663936).
//! - [Weighted Distributed Hash Tables](http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.414.9353&rep=rep1&type=pdf)
//!   > Schindelhauer, Christian, and Gunnar Schomaker. 2005. “Weighted Distributed Hash Tables.” In *Proceedings of the Seventeenth Annual Acm Symposium on Parallelism in Algorithms and Architectures*, 218–27. SPAA ’05. New York, NY, USA: ACM. doi:[10.1145/1073970.1074008](https://doi.org/10.1145/1073970.1074008).
//!
//! ## License
//!
//! `hash-rings-rs` is dual-licensed under the terms of either the MIT License or the Apache License
//! (Version 2.0).
//!
//! See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for more details.

#![warn(missing_docs)]

pub mod carp;
pub mod consistent;
pub mod jump;
pub mod maglev;
pub mod mpc;
pub mod rendezvous;
mod util;
pub mod weighted_rendezvous;
