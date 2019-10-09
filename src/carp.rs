//! Hashing ring implemented using the Cache Ring Routing Protocol.

use crate::util;
use std::collections::hash_map::RandomState;
use std::f64;
use std::hash::{BuildHasher, Hash};

/// A node with an associated weight.
///
/// The distribution of points to nodes is proportional to the weights of the nodes. For example, a
/// node with a weight of 3 will receive approximately three times more points than a node with a
/// weight of 1.
pub struct Node<'a, T> {
    id: &'a T,
    hash: u64,
    weight: f64,
    relative_weight: f64,
}

impl<'a, T> Node<'a, T> {
    /// Constructs a new node with a particular weight associated with it.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::{Node, Ring};
    ///
    /// let node = Node::new(&"node-1", 1f64);
    /// ```
    pub fn new(id: &'a T, weight: f64) -> Self {
        Node {
            id,
            hash: 0,
            weight,
            relative_weight: 0f64,
        }
    }
}

/// A hashing ring implemented using the Cache Array Routing Protocol.
///
/// The Cache Array Routing Protocol calculates the relative weight for each node in the ring to
/// distribute points according to their weights.
///
/// # Examples
/// ```
/// use hash_rings::carp::{Node, Ring};
/// use std::collections::hash_map::DefaultHasher;
/// use std::hash::BuildHasherDefault;
///
/// type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;
///
/// let mut ring = Ring::with_hasher(
///     DefaultBuildHasher::default(),
///     vec![Node::new(&"node-1", 1f64), Node::new(&"node-2", 3f64)],
/// );
///
/// ring.remove_node(&"node-1");
///
/// assert_eq!(ring.get_node(&"point-1"), &"node-2");
/// assert_eq!(ring.len(), 1);
///
/// let mut iterator = ring.iter();
/// assert_eq!(iterator.next(), Some((&"node-2", 3f64)));
/// assert_eq!(iterator.next(), None);
/// ```
pub struct Ring<'a, T, H = RandomState> {
    nodes: Vec<Node<'a, T>>,
    hash_builder: H,
}

impl<'a, T> Ring<'a, T, RandomState> {
    /// Constructs a new, empty `Ring<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new(vec![]);
    /// ```
    pub fn new(nodes: Vec<Node<'a, T>>) -> Self
    where
        T: Hash + Ord,
    {
        Self::with_hasher(Default::default(), nodes)
    }
}

impl<'a, T, H> Ring<'a, T, H> {
    fn rebalance(&mut self) {
        let mut product = 1f64;
        let len = self.nodes.len() as f64;
        for i in 0..self.nodes.len() {
            let index = i as f64;
            let mut res;
            if i == 0 {
                res = (len * self.nodes[i].weight).powf(1f64 / len);
            } else {
                res = (len - index) * (self.nodes[i].weight - self.nodes[i - 1].weight) / product;
                res += self.nodes[i - 1].relative_weight.powf(len - index);
                res = res.powf(1f64 / (len - index));
            }

            product *= res;
            self.nodes[i].relative_weight = res;
        }
        if let Some(max_relative_weight) = self.nodes.last().map(|node| node.relative_weight) {
            for node in &mut self.nodes {
                node.relative_weight /= max_relative_weight
            }
        }
    }

    /// Constructs a new, empty `Ring<T>` with a specified hash builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::Ring;
    /// use std::collections::hash_map::DefaultHasher;
    /// use std::hash::BuildHasherDefault;
    ///
    /// type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;
    ///
    /// let mut ring: Ring<&str, _> = Ring::with_hasher(DefaultBuildHasher::default(), vec![]);
    /// ```
    pub fn with_hasher(hash_builder: H, mut nodes: Vec<Node<'a, T>>) -> Self
    where
        T: Hash + Ord,
        H: BuildHasher + Default,
    {
        for node in &mut nodes {
            node.hash = util::gen_hash(&hash_builder, node.id);
        }
        nodes.reverse();
        nodes.sort_by_key(|node| node.id);
        nodes.dedup_by_key(|node| node.id);
        nodes.sort_by(|n, m| {
            if (n.weight - m.weight).abs() < f64::EPSILON {
                n.id.cmp(m.id)
            } else {
                n.weight
                    .partial_cmp(&m.weight)
                    .expect("Expected all non-NaN floats.")
            }
        });
        let mut ret = Self {
            nodes,
            hash_builder,
        };
        ret.rebalance();
        ret
    }

    /// Inserts a node into the ring with a particular weight.
    ///
    /// Increasing the weight will increase the number of expected points mapped to the node. For
    /// example, a node with a weight of three will receive approximately three times more points
    /// than a node with a weight of one.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::{Node, Ring};
    ///
    /// let mut ring = Ring::new(vec![Node::new(&"node-1", 1f64)]);
    ///
    /// ring.insert_node(Node::new(&"node-2", 1f64));
    /// ```
    pub fn insert_node(&mut self, mut new_node: Node<'a, T>)
    where
        T: Hash + Ord,
        H: BuildHasher,
    {
        new_node.hash = util::gen_hash(&self.hash_builder, new_node.id);
        if let Some(index) = self.nodes.iter().position(|node| node.id == new_node.id) {
            self.nodes[index] = new_node;
        } else {
            self.nodes.push(new_node);
        }
        self.nodes.sort_by(|n, m| {
            if (n.weight - m.weight).abs() < f64::EPSILON {
                n.id.cmp(m.id)
            } else {
                n.weight
                    .partial_cmp(&m.weight)
                    .expect("Expected all non-NaN floats.")
            }
        });
        self.rebalance();
    }

    /// Removes a node from the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::{Node, Ring};
    ///
    /// let mut ring = Ring::new(vec![Node::new(&"node-1", 1f64), Node::new(&"node-2", 3f64)]);
    ///
    /// ring.remove_node(&"node-2");
    /// ```
    pub fn remove_node(&mut self, id: &T)
    where
        T: Eq,
    {
        if let Some(index) = self.nodes.iter().position(|node| node.id == id) {
            self.nodes.remove(index);
            self.rebalance();
        }
    }

    /// Returns the node associated with a point.
    ///
    /// # Panics
    ///
    /// Panics if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::{Node, Ring};
    ///
    /// let mut ring = Ring::new(vec![Node::new(&"node-1", 1f64)]);
    ///
    /// assert_eq!(ring.get_node(&"point-1"), &"node-1");
    /// ```
    pub fn get_node<U>(&self, point: &U) -> &'a T
    where
        T: Ord,
        U: Hash,
        H: BuildHasher,
    {
        let point_hash = util::gen_hash(&self.hash_builder, point);
        self.nodes
            .iter()
            .map(|node| {
                (
                    util::combine_hash(&self.hash_builder, node.hash, point_hash) as f64
                        * node.relative_weight,
                    node.id,
                )
            })
            .max_by(|n, m| {
                if n == m {
                    n.1.cmp(m.1)
                } else {
                    n.0.partial_cmp(&m.0).expect("Expected all non-NaN floats.")
                }
            })
            .expect("Expected non-empty ring.")
            .1
    }

    /// Returns the number of nodes in the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::{Node, Ring};
    ///
    /// let mut ring = Ring::new(vec![Node::new(&"node-1", 1f64), Node::new(&"node-2", 3f64)]);
    ///
    /// assert_eq!(ring.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::{Node, Ring};
    ///
    /// let ring: Ring<'_, u32, _> = Ring::new(vec![]);
    ///
    /// assert!(ring.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns an iterator over the ring. The iterator will yield nodes and their weights in
    /// sorted by weight, and then by id.
    /// particular order.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::carp::{Node, Ring};
    ///
    /// let mut ring = Ring::new(vec![Node::new(&"node-1", 1f64), Node::new(&"node-2", 3f64)]);
    ///
    /// let mut iterator = ring.iter();
    /// assert_eq!(iterator.next(), Some((&"node-1", 1f64)));
    /// assert_eq!(iterator.next(), Some((&"node-2", 3f64)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&'a self) -> impl Iterator<Item = (&'a T, f64)> {
        self.nodes.iter().map(|node| (&*node.id, node.weight))
    }
}

impl<'a, T, H> IntoIterator for &'a Ring<'a, T, H> {
    type IntoIter = Box<dyn Iterator<Item = (&'a T, f64)> + 'a>;
    type Item = (&'a T, f64);

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Ring};
    use crate::test_util::BuildDefaultHasher;

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {{
            let (a, b) = (&$a, &$b);
            assert!(
                (*a - *b).abs() < 1.0e-6,
                "{} is not approximately equal to {}",
                *a,
                *b
            );
        }};
    }

    #[test]
    fn test_size_empty() {
        let ring: Ring<'_, u32, _> = Ring::with_hasher(BuildDefaultHasher::default(), vec![]);
        assert!(ring.is_empty());
        assert_eq!(ring.len(), 0);
    }

    #[test]
    fn test_correct_weights() {
        let ring = Ring::with_hasher(
            BuildDefaultHasher::default(),
            vec![Node::new(&0, 0.4), Node::new(&1, 0.4), Node::new(&2, 0.2)],
        );
        assert_eq!(ring.nodes[0].id, &2);
        assert_eq!(ring.nodes[1].id, &0);
        assert_eq!(ring.nodes[2].id, &1);
        assert_approx_eq!(ring.nodes[0].relative_weight, 0.774_596);
        assert_approx_eq!(ring.nodes[1].relative_weight, 1.000_000);
        assert_approx_eq!(ring.nodes[2].relative_weight, 1.000_000);
    }

    #[test]
    fn test_new_replace() {
        let ring = Ring::with_hasher(
            BuildDefaultHasher::default(),
            vec![Node::new(&0, 0.5), Node::new(&1, 0.1), Node::new(&1, 0.5)],
        );

        assert_eq!(ring.nodes[0].id, &0);
        assert_eq!(ring.nodes[1].id, &1);
        assert_approx_eq!(ring.nodes[0].relative_weight, 1.000_000);
        assert_approx_eq!(ring.nodes[1].relative_weight, 1.000_000);
    }

    #[test]
    fn test_insert_node() {
        let mut ring = Ring::with_hasher(BuildDefaultHasher::default(), vec![Node::new(&0, 0.5)]);
        ring.insert_node(Node::new(&1, 0.5));

        assert_eq!(ring.nodes[0].id, &0);
        assert_eq!(ring.nodes[1].id, &1);
        assert_approx_eq!(ring.nodes[0].relative_weight, 1.000_000);
        assert_approx_eq!(ring.nodes[1].relative_weight, 1.000_000);
    }

    #[test]
    fn test_insert_node_replace() {
        let mut ring = Ring::with_hasher(
            BuildDefaultHasher::default(),
            vec![Node::new(&0, 0.5), Node::new(&1, 0.1)],
        );
        ring.insert_node(Node::new(&1, 0.5));

        assert_eq!(ring.nodes[0].id, &0);
        assert_eq!(ring.nodes[1].id, &1);
        assert_approx_eq!(ring.nodes[0].relative_weight, 1.000_000);
        assert_approx_eq!(ring.nodes[1].relative_weight, 1.000_000);
    }

    #[test]
    fn test_remove_node() {
        let mut ring = Ring::with_hasher(
            BuildDefaultHasher::default(),
            vec![Node::new(&0, 0.5), Node::new(&1, 0.5), Node::new(&2, 0.1)],
        );
        ring.remove_node(&2);

        assert_eq!(ring.nodes[0].id, &0);
        assert_eq!(ring.nodes[1].id, &1);
        assert_approx_eq!(ring.nodes[0].relative_weight, 1.000_000);
        assert_approx_eq!(ring.nodes[1].relative_weight, 1.000_000);
    }

    #[test]
    fn test_get_node() {
        let ring = Ring::with_hasher(
            BuildDefaultHasher::default(),
            vec![Node::new(&0, 1.0), Node::new(&1, 1.0)],
        );

        assert_eq!(ring.get_node(&0), &0);
        assert_eq!(ring.get_node(&1), &0);
        assert_eq!(ring.get_node(&2), &0);
        assert_eq!(ring.get_node(&3), &1);
        assert_eq!(ring.get_node(&4), &1);
        assert_eq!(ring.get_node(&5), &1);
    }

    #[test]
    fn test_iter() {
        let ring = Ring::with_hasher(
            BuildDefaultHasher::default(),
            vec![Node::new(&0, 0.4), Node::new(&1, 0.4), Node::new(&2, 0.2)],
        );

        let mut iterator = ring.iter();
        let mut node;

        node = iterator.next().unwrap();
        assert_eq!(node.0, &2);
        assert_approx_eq!(node.1, 0.2);

        node = iterator.next().unwrap();
        assert_eq!(node.0, &0);
        assert_approx_eq!(node.1, 0.4);

        node = iterator.next().unwrap();
        assert_eq!(node.0, &1);
        assert_approx_eq!(node.1, 0.4);

        assert_eq!(iterator.next(), None);
    }
}
