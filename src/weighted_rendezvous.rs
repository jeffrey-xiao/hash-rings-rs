//! Hashing ring implemented using weighted rendezvous hashing.

use crate::util;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hash};
use std::vec::Vec;

/// A hashing ring implemented using weighted rendezvous hashing.
///
/// Rendezvous hashing is based on based on assigning a pseudorandom value to node-point pair.
/// A point is mapped to the node that yields the greatest value associated with the node-point
/// pair.
///
/// # Examples
/// ```
/// use hash_rings::weighted_rendezvous::Ring;
/// use std::collections::hash_map::DefaultHasher;
/// use std::hash::BuildHasherDefault;
///
/// type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;
///
/// let mut ring = Ring::new();
///
/// ring.insert_node(&"node-1", 1f64);
/// ring.insert_node(&"node-2", 3f64);
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
    nodes: HashMap<&'a T, f64>,
    hash_builder: H,
}

impl<'a, T> Ring<'a, T, RandomState> {
    /// Constructs a new, empty `Ring<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    /// ```
    pub fn new() -> Self
    where
        T: Hash + Eq,
    {
        Self::default()
    }
}

impl<'a, T, H> Ring<'a, T, H> {
    /// Constructs a new, empty `Ring<T>` with a specified hash builder;
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Ring;
    /// use std::collections::hash_map::DefaultHasher;
    /// use std::hash::BuildHasherDefault;
    ///
    /// type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;
    ///
    /// let mut ring: Ring<&str, _> = Ring::with_hasher(DefaultBuildHasher::default());
    /// ```
    pub fn with_hasher(hash_builder: H) -> Self
    where
        T: Hash + Eq,
        H: BuildHasher,
    {
        Self {
            nodes: HashMap::new(),
            hash_builder,
        }
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
    /// use hash_rings::weighted_rendezvous::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// // "node-2" will receive three times more points than "node-1"
    /// ring.insert_node(&"node-1", 1f64);
    /// ring.insert_node(&"node-2", 3f64);
    /// ```
    pub fn insert_node(&mut self, id: &'a T, weight: f64)
    where
        T: Hash + Eq,
    {
        self.nodes.insert(id, weight);
    }

    /// Removes a node from the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// ring.insert_node(&"node-1", 1f64);
    /// ring.insert_node(&"node-2", 1f64);
    /// ring.remove_node(&"node-2");
    /// ```
    pub fn remove_node(&mut self, id: &T)
    where
        T: Hash + Eq,
    {
        self.nodes.remove(id);
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
    /// use hash_rings::weighted_rendezvous::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// ring.insert_node(&"node-1", 1f64);
    /// assert_eq!(ring.get_node(&"point-1"), &"node-1");
    /// ```
    pub fn get_node<U>(&self, key: &U) -> &'a T
    where
        T: Hash + Ord,
        U: Hash,
        H: BuildHasher,
    {
        let point_hash = util::gen_hash(&self.hash_builder, key);
        self.nodes
            .iter()
            .map(|entry| {
                let hash = util::combine_hash(
                    &self.hash_builder,
                    util::gen_hash(&self.hash_builder, entry.0),
                    point_hash,
                );
                (
                    -entry.1 / (hash as f64 / u64::max_value() as f64).ln(),
                    entry.0,
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
    /// use hash_rings::weighted_rendezvous::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// ring.insert_node(&"node-1", 3f64);
    /// assert_eq!(ring.len(), 1);
    /// ```
    pub fn len(&self) -> usize
    where
        T: Hash + Eq,
    {
        self.nodes.len()
    }

    /// Returns `true` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// assert!(ring.is_empty());
    /// ring.insert_node(&"node-1", 3f64);
    /// assert!(!ring.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool
    where
        T: Hash + Eq,
    {
        self.nodes.is_empty()
    }

    /// Returns an iterator over the ring. The iterator will yield nodes and their weights in no
    /// particular order.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Ring;
    ///
    /// let mut ring = Ring::new();
    /// ring.insert_node(&"node-1", 1f64);
    ///
    /// let mut iterator = ring.iter();
    /// assert_eq!(iterator.next(), Some((&"node-1", 1f64)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&'a self) -> impl Iterator<Item = (&'a T, f64)>
    where
        T: Hash + Eq,
    {
        self.nodes.iter().map(|node_entry| {
            let (id, weight) = node_entry;
            (&**id, *weight)
        })
    }
}

impl<'a, T, H> IntoIterator for &'a Ring<'a, T, H>
where
    T: Hash + Eq,
{
    type IntoIter = Box<dyn Iterator<Item = (&'a T, f64)> + 'a>;
    type Item = (&'a T, f64);

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl<'a, T, H> Default for Ring<'a, T, H>
where
    T: Hash + Eq,
    H: BuildHasher + Default,
{
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

/// A client that uses `Ring<T>`.
///
/// # Examples
/// ```
/// use hash_rings::weighted_rendezvous::Client;
///
/// let mut client = Client::new();
/// client.insert_node(&"node-1", 3f64);
/// client.insert_point(&"point-1");
/// client.insert_point(&"point-2");
///
/// assert_eq!(client.len(), 1);
/// assert_eq!(client.get_node(&"point-1"), &"node-1");
///
/// client.remove_point(&"point-2");
/// assert_eq!(client.get_points(&"node-1"), [&"point-1"]);
/// ```
pub struct Client<'a, T, U, H = RandomState> {
    ring: Ring<'a, T, H>,
    nodes: HashMap<&'a T, HashSet<&'a U>>,
    points: HashMap<&'a U, (&'a T, f64)>,
    hash_builder: H,
}

impl<'a, T, U> Client<'a, T, U, RandomState> {
    /// Constructs a new, empty `Client<T, U>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    /// ```
    pub fn new() -> Self
    where
        T: Hash + Eq,
        U: Hash + Eq,
    {
        Self::default()
    }
}

impl<'a, T, U, H> Client<'a, T, U, H> {
    /// Constructs a new, empty `Client<T, U>` with a specified hash builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    /// use std::collections::hash_map::DefaultHasher;
    /// use std::hash::BuildHasherDefault;
    ///
    /// type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;
    ///
    /// let mut client: Client<&str, &str, _> = Client::with_hasher(DefaultBuildHasher::default());
    /// ```
    pub fn with_hasher(hash_builder: H) -> Self
    where
        T: Hash + Eq,
        U: Hash + Eq,
        H: BuildHasher + Clone,
    {
        Self {
            ring: Ring::with_hasher(hash_builder.clone()),
            nodes: HashMap::new(),
            points: HashMap::new(),
            hash_builder,
        }
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
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// // "node-2" will receive three times more points than "node-1"
    /// client.insert_node(&"node-1", 1f64);
    /// client.insert_node(&"node-2", 3f64);
    /// ```
    pub fn insert_node(&mut self, id: &'a T, weight: f64)
    where
        T: Hash + Eq,
        U: Hash + Eq,
        H: BuildHasher,
    {
        self.ring.insert_node(id, weight);

        let mut new_points = HashSet::new();

        for (point, node_entry) in &mut self.points {
            let (ref mut original_node, ref mut original_score) = node_entry;
            let point_hash = util::gen_hash(&self.hash_builder, point);
            let curr_hash = util::combine_hash(
                &self.hash_builder,
                util::gen_hash(&self.hash_builder, id),
                point_hash,
            );
            let curr_score = -weight / (curr_hash as f64 / u64::max_value() as f64).ln();

            if curr_score > *original_score {
                self.nodes
                    .get_mut(original_node)
                    .expect("Expected node to exist.")
                    .remove(point);
                new_points.insert(*point);
                *original_score = curr_score;
                *original_node = id;
            }
        }

        self.nodes.insert(id, new_points);
    }

    /// Removes a node from the ring.
    ///
    /// # Panics
    ///
    /// Panics if the ring is empty after removal of a node or if the node does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// client.insert_node(&"node-1", 1f64);
    /// client.insert_node(&"node-2", 1f64);
    /// client.remove_node(&"node-1");
    /// ```
    pub fn remove_node(&mut self, id: &T)
    where
        T: Hash + Ord,
        U: Hash + Eq,
        H: BuildHasher,
    {
        self.ring.remove_node(id);
        if self.ring.is_empty() {
            panic!("Error: empty ring after deletion.");
        }
        if let Some(points) = self.nodes.remove(id) {
            for point in points {
                let new_node = self.ring.get_node(point);
                let point_hash = util::gen_hash(&self.hash_builder, point);
                let curr_hash = util::combine_hash(
                    &self.hash_builder,
                    util::gen_hash(&self.hash_builder, new_node),
                    point_hash,
                );
                let coefficient = -1.0 / (curr_hash as f64 / u64::max_value() as f64).ln();
                let curr_score = self.ring.nodes[new_node] / coefficient;

                self.nodes
                    .get_mut(new_node)
                    .expect("Expected node to exist.")
                    .insert(point);
                self.points.insert(point, (new_node, curr_score));
            }
        }
    }

    /// Returns the points associated with a node.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// client.insert_node(&"node-1", 1f64);
    /// client.insert_point(&"point-1");
    /// assert_eq!(client.get_points(&"node-1"), [&"point-1"]);
    /// ```
    pub fn get_points(&mut self, id: &T) -> Vec<&U>
    where
        T: Hash + Eq,
        U: Hash + Eq,
    {
        self.nodes[id].iter().cloned().collect()
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
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// client.insert_node(&"node-1", 1f64);
    /// client.insert_point(&"point-1");
    /// assert_eq!(client.get_node(&"point-1"), &"node-1");
    /// ```
    pub fn get_node(&mut self, key: &U) -> &T
    where
        T: Hash + Ord,
        U: Hash,
        H: BuildHasher,
    {
        self.ring.get_node(key)
    }

    /// Inserts a point into the ring.
    ///
    /// # Panics
    ///
    /// Panics if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client = Client::new();
    /// client.insert_node(&"node-1", 1f64);
    /// client.insert_point(&"point-1");
    /// ```
    pub fn insert_point(&mut self, point: &'a U)
    where
        T: Hash + Ord,
        U: Hash + Eq,
        H: BuildHasher,
    {
        let new_node = self.ring.get_node(point);
        let point_hash = util::gen_hash(&self.hash_builder, point);
        let curr_hash = util::combine_hash(
            &self.hash_builder,
            util::gen_hash(&self.hash_builder, new_node),
            point_hash,
        );
        let coefficient = -1.0 / (curr_hash as f64 / u64::max_value() as f64).ln();
        let curr_score = self.ring.nodes[new_node] / coefficient;

        self.nodes
            .get_mut(new_node)
            .expect("Expected node to exist.")
            .insert(point);
        self.points.insert(point, (new_node, curr_score));
    }

    /// Removes a point from the ring.
    ///
    /// # Panics
    ///
    /// Panics if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client = Client::new();
    /// client.insert_node(&"node-1", 1f64);
    /// client.insert_point(&"point-1");
    /// client.remove_point(&"point-1");
    /// ```
    pub fn remove_point(&mut self, point: &U)
    where
        T: Hash + Ord,
        U: Hash + Eq,
        H: BuildHasher,
    {
        let node = self.ring.get_node(point);
        self.nodes
            .get_mut(node)
            .expect("Expected node to exist.")
            .remove(point);
        self.points.remove(point);
    }

    /// Returns the number of nodes in the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// client.insert_node(&"node-1", 3f64);
    /// assert_eq!(client.len(), 1);
    /// ```
    pub fn len(&self) -> usize
    where
        T: Hash + Eq,
    {
        self.nodes.len()
    }

    /// Returns `true` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// assert!(client.is_empty());
    /// client.insert_node(&"node-1", 3f64);
    /// assert!(!client.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool
    where
        T: Hash + Eq,
    {
        self.ring.is_empty()
    }

    /// Returns an iterator over the ring. The iterator will yield nodes and points in no
    /// particular order.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::weighted_rendezvous::Client;
    ///
    /// let mut client = Client::new();
    /// client.insert_node(&"node-1", 1f64);
    /// client.insert_point(&"point-1");
    ///
    /// let mut iterator = client.iter();
    /// assert_eq!(iterator.next(), Some((&"node-1", vec![&"point-1"])));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&'a self) -> impl Iterator<Item = (&'a T, Vec<&'a U>)>
    where
        T: Hash + Eq,
        U: Hash + Eq,
    {
        self.nodes.iter().map(move |node_entry| {
            let (node_id, points) = node_entry;
            (&**node_id, points.iter().cloned().collect())
        })
    }
}

impl<'a, T, U, H> IntoIterator for &'a Client<'a, T, U, H>
where
    T: Hash + Eq,
    U: Hash + Eq,
    H: BuildHasher + Default,
{
    type IntoIter = Box<dyn Iterator<Item = (&'a T, Vec<&'a U>)> + 'a>;
    type Item = (&'a T, Vec<&'a U>);

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl<'a, T, U, H> Default for Client<'a, T, U, H>
where
    T: Hash + Eq,
    U: Hash + Eq,
    H: BuildHasher + Default + Clone,
{
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{Client, Ring};
    use crate::test_util::BuildDefaultHasher;

    #[test]
    fn test_size_empty() {
        let client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        assert!(client.is_empty());
        assert_eq!(client.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_panic_remove_node_empty_client() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 1f64);
        client.remove_node(&0);
    }

    #[test]
    #[should_panic]
    fn test_panic_remove_node_non_existent_node() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.remove_node(&0);
    }

    #[test]
    #[should_panic]
    fn test_panic_get_node_empty_client() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.get_node(&0);
    }

    #[test]
    #[should_panic]
    fn test_panic_insert_point_empty_client() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_point(&0);
    }

    #[test]
    #[should_panic]
    fn test_panic_remove_point_empty_client() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.remove_point(&0);
    }

    #[test]
    fn test_insert_node() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 0f64);
        client.insert_point(&0);
        client.insert_node(&1, 1f64);
        assert_eq!(client.get_points(&1).as_slice(), [&0u32]);
    }

    #[test]
    fn test_remove_node() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 1f64);
        client.insert_point(&0);
        client.insert_point(&1);
        client.insert_point(&2);
        client.insert_node(&1, 1f64);
        client.remove_node(&1);

        let points = client.get_points(&0);

        assert!(points.contains(&&0u32));
        assert!(points.contains(&&1u32));
        assert!(points.contains(&&2u32));
    }

    #[test]
    fn test_get_node() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 3f64);
        client.insert_node(&1, 0f64);
        client.insert_node(&2, 0f64);
        assert_eq!(client.get_node(&0), &0);
    }

    #[test]
    fn test_insert_point() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 3f64);
        client.insert_point(&0);
        assert_eq!(client.get_points(&0).as_slice(), [&0u32]);
    }

    #[test]
    fn test_remove_point() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 3f64);
        client.insert_point(&0);
        client.remove_point(&0);
        let expected: [&u32; 0] = [];
        assert_eq!(client.get_points(&0).as_slice(), expected);
    }

    #[test]
    fn test_iter() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 3f64);
        client.insert_point(&1);
        client.insert_point(&2);
        client.insert_point(&3);
        client.insert_point(&4);
        client.insert_point(&5);
        let mut actual: Vec<(&u32, Vec<&u32>)> = client.iter().collect();
        actual[0].1.sort();
        assert_eq!(actual[0].0, &0);
        assert_eq!(actual[0].1.as_slice(), [&1, &2, &3, &4, &5]);
    }

    #[test]
    fn test_ring_len() {
        let mut ring = Ring::with_hasher(BuildDefaultHasher::default());

        ring.insert_node(&0, 1f64);
        assert_eq!(ring.len(), 1);
    }

    #[test]
    fn test_ring_iter() {
        let mut ring: Ring<'_, u32, BuildDefaultHasher> = Ring::default();

        ring.insert_node(&0, 1.0f64);
        let mut iterator = ring.iter();
        assert_eq!(iterator.next(), Some((&0, 1.0f64)));
        assert_eq!(iterator.next(), None);
    }
}
