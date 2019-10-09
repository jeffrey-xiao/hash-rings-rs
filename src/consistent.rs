//! Hashing ring implemented using consistent hashing.

use crate::util;
use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{BuildHasher, Hash};
use std::iter::Iterator;
use std::vec::Vec;

/// A hashing ring implemented using consistent hashing.
///
/// Consistent hashing is based on mapping each node to a pseudorandom value. In this
/// implementation the pseudorandom is a combination of the hash of the node and the hash of the
/// replica number. A point is also represented as a pseudorandom value and it is mapped to the
/// node with the smallest value that is greater than or equal to the point's value. If such a
/// node does not exist, then the point maps to the node with the smallest value.
///
/// # Examples
/// ```
/// use hash_rings::consistent::Ring;
/// use std::collections::hash_map::DefaultHasher;
/// use std::hash::BuildHasherDefault;
///
/// type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;
///
/// let mut ring = Ring::with_hasher(DefaultBuildHasher::default());
///
/// ring.insert_node(&"node-1", 1);
/// ring.insert_node(&"node-2", 3);
///
/// ring.remove_node(&"node-1");
///
/// assert_eq!(ring.get_node(&"point-1"), &"node-2");
/// assert_eq!(ring.len(), 1);
///
/// let mut iterator = ring.iter();
/// assert_eq!(iterator.next(), Some((&"node-2", 3)));
/// assert_eq!(iterator.next(), None);
/// ```
pub struct Ring<'a, T, H = RandomState> {
    nodes: BTreeMap<u64, &'a T>,
    replicas: HashMap<&'a T, usize>,
    hash_builder: H,
}

impl<'a, T> Ring<'a, T, RandomState> {
    /// Constructs a new, empty `Ring<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Ring;
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
    /// Constructs a new, empty `Ring<T>` with a specified hash builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Ring;
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
        H: BuildHasher + Default,
    {
        Self {
            nodes: BTreeMap::new(),
            replicas: HashMap::new(),
            hash_builder,
        }
    }

    fn get_next_node(&self, hash: u64) -> Option<&T> {
        self.nodes
            .range(hash..)
            .next()
            .or_else(|| self.nodes.iter().next())
            .map(|entry| *entry.1)
    }

    /// Inserts a node into the ring with a number of replicas.
    ///
    /// Increasing the number of replicas will increase the number of expected points mapped to the
    /// node. For example, a node with three replicas will receive approximately three times more
    /// points than a node with one replica.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// // "node-2" will receive three times more points than "node-1"
    /// ring.insert_node(&"node-1", 1);
    /// ring.insert_node(&"node-2", 3);
    /// ```
    pub fn insert_node(&mut self, id: &'a T, replicas: usize)
    where
        T: Hash + Eq,
        H: BuildHasher,
    {
        for i in 0..replicas {
            let hash = util::combine_hash(
                &self.hash_builder,
                util::gen_hash(&self.hash_builder, id),
                util::gen_hash(&self.hash_builder, &i),
            );
            self.nodes.insert(hash, id);
        }
        self.replicas.insert(id, replicas);
    }

    /// Removes a node and all its replicas from the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// ring.insert_node(&"node-1", 1);
    /// ring.insert_node(&"node-2", 1);
    /// ring.remove_node(&"node-2");
    /// ```
    pub fn remove_node(&mut self, id: &T)
    where
        T: Hash + Eq,
        H: BuildHasher,
    {
        for i in 0..self.replicas[id] {
            let hash = util::combine_hash(
                &self.hash_builder,
                util::gen_hash(&self.hash_builder, id),
                util::gen_hash(&self.hash_builder, &i),
            );
            let should_remove = {
                if let Some(existing_id) = self.nodes.get(&hash) {
                    *existing_id == id
                } else {
                    false
                }
            };

            if should_remove {
                self.nodes.remove(&hash);
            }
        }
        self.replicas.remove(id);
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
    /// use hash_rings::consistent::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// ring.insert_node(&"node-1", 1);
    /// assert_eq!(ring.get_node(&"point-1"), &"node-1");
    /// ```
    pub fn get_node<U>(&mut self, point: &U) -> &T
    where
        U: Hash,
        H: BuildHasher,
    {
        let hash = util::gen_hash(&self.hash_builder, point);
        if let Some(node) = self.get_next_node(hash) {
            &*node
        } else {
            panic!("Error: empty ring.");
        }
    }

    fn contains_node(&self, index: u64) -> bool {
        self.nodes.contains_key(&index)
    }

    fn get_replica_count(&self, id: &T) -> usize
    where
        T: Hash + Eq,
    {
        self.replicas[id]
    }

    /// Returns the number of nodes in the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// ring.insert_node(&"node-1", 3);
    /// assert_eq!(ring.len(), 1);
    /// ```
    pub fn len(&self) -> usize
    where
        T: Hash + Eq,
    {
        self.replicas.len()
    }

    /// Returns `true` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new();
    ///
    /// assert!(ring.is_empty());
    /// ring.insert_node(&"node-1", 3);
    /// assert!(!ring.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool
    where
        T: Hash + Eq,
    {
        self.replicas.is_empty()
    }

    /// Returns an iterator over the ring. The iterator will yield nodes and the replica count in
    /// no particular order.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Ring;
    ///
    /// let mut ring = Ring::new();
    /// ring.insert_node(&"node-1", 1);
    ///
    /// let mut iterator = ring.iter();
    /// assert_eq!(iterator.next(), Some((&"node-1", 1)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&'a self) -> impl Iterator<Item = (&'a T, usize)>
    where
        T: Hash + Eq,
    {
        self.replicas.iter().map(|replica| {
            let (id, replica_count) = replica;
            (&**id, *replica_count)
        })
    }
}

impl<'a, T, H> IntoIterator for &'a Ring<'a, T, H>
where
    T: Hash + Eq,
{
    type IntoIter = Box<dyn Iterator<Item = (&'a T, usize)> + 'a>;
    type Item = (&'a T, usize);

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
/// use hash_rings::consistent::Client;
///
/// let mut client = Client::new();
/// client.insert_node(&"node-1", 3);
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
    data: BTreeMap<u64, HashSet<&'a U>>,
}

impl<'a, T, U> Client<'a, T, U, RandomState> {
    /// Constructs a new, empty `Client<T, U>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Client;
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
    /// use hash_rings::consistent::Client;
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
        H: BuildHasher + Default,
    {
        Self {
            ring: Ring::with_hasher(hash_builder),
            data: BTreeMap::new(),
        }
    }

    fn get_next_node(&mut self, hash: u64) -> Option<(u64, &mut HashSet<&'a U>)> {
        if self.data.range_mut(hash..).next().is_some() {
            self.data
                .range_mut(hash..)
                .next()
                .map(|entry| (*entry.0, entry.1))
        } else if self.data.iter_mut().next().is_some() {
            self.data.iter_mut().next().map(|entry| (*entry.0, entry.1))
        } else {
            None
        }
    }

    /// Inserts a node into the ring with a number of replicas.
    ///
    /// Increasing the number of replicas will increase the number of expected points mapped to the
    /// node. For example, a node with three replicas will receive approximately three times more
    /// points than a node with one replica.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// // "node-2" will receive three times more points than "node-1"
    /// client.insert_node(&"node-1", 1);
    /// client.insert_node(&"node-2", 3);
    /// ```
    pub fn insert_node(&mut self, id: &'a T, replicas: usize)
    where
        T: Hash + Eq,
        U: Hash + Eq,
        H: BuildHasher,
    {
        let new_hashes = (0..replicas)
            .map(|replica| {
                util::combine_hash(
                    &self.ring.hash_builder,
                    util::gen_hash(&self.ring.hash_builder, &id),
                    util::gen_hash(&self.ring.hash_builder, &replica),
                )
            })
            .collect::<Vec<u64>>();
        self.ring.insert_node(id, replicas);
        for new_hash in new_hashes {
            // if hash already exists, then no additional work is needed to be done
            if self.data.contains_key(&new_hash) {
                continue;
            }
            let hash = match self.get_next_node(new_hash) {
                Some((hash, _)) => hash,
                None => {
                    self.data.insert(new_hash, HashSet::new());
                    continue;
                },
            };
            let Client { ring, data } = self;
            let (old_set, new_set) = data
                .get_mut(&hash)
                .expect("Expected node to exist.")
                .drain()
                .partition(|point| {
                    let point_hash = util::gen_hash(&ring.hash_builder, point);
                    if new_hash < hash {
                        new_hash < point_hash && point_hash < hash
                    } else {
                        new_hash < point_hash || point_hash < hash
                    }
                });
            self.data.insert(hash, old_set);
            self.data.insert(new_hash, new_set);
        }
    }

    /// Removes a node and all its replicas from the ring.
    ///
    /// # Panics
    ///
    /// Panics if the ring is empty after removal of a node or if the node does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// client.insert_node(&"node-1", 1);
    /// client.insert_node(&"node-2", 1);
    /// client.remove_node(&"node-2");
    /// ```
    pub fn remove_node(&mut self, id: &T)
    where
        T: Hash + Eq,
        U: Hash + Eq,
        H: BuildHasher,
    {
        let replicas = self.ring.get_replica_count(id);
        self.ring.remove_node(id);
        for i in 0..replicas {
            let hash = util::combine_hash(
                &self.ring.hash_builder,
                util::gen_hash(&self.ring.hash_builder, id),
                util::gen_hash(&self.ring.hash_builder, &i),
            );
            if !self.ring.contains_node(hash) {
                if let Some(points) = self.data.remove(&hash) {
                    if let Some((_, next_points)) = self.get_next_node(hash) {
                        next_points.extend(points);
                    } else {
                        panic!("Error: empty ring after deletion.");
                    }
                }
            }
        }
    }

    /// Returns the points associated with a node and its replicas.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// client.insert_node(&"node-1", 1);
    /// client.insert_point(&"point-1");
    /// assert_eq!(client.get_points(&"node-1"), [&"point-1"]);
    /// ```
    pub fn get_points(&self, id: &T) -> Vec<&U>
    where
        T: Hash + Eq,
        U: Hash + Eq,
        H: BuildHasher,
    {
        let mut ret: Vec<&U> = Vec::new();
        for i in 0..self.ring.get_replica_count(id) {
            let hash = util::combine_hash(
                &self.ring.hash_builder,
                util::gen_hash(&self.ring.hash_builder, id),
                util::gen_hash(&self.ring.hash_builder, &i),
            );
            if let Some(points) = self.data.get(&hash) {
                ret.extend(points.iter());
            }
        }
        ret
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
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// client.insert_node(&"node-1", 1);
    /// client.insert_point(&"point-1");
    /// assert_eq!(client.get_node(&"point-1"), &"node-1");
    /// ```
    pub fn get_node(&mut self, point: &U) -> &T
    where
        U: Hash + Eq,
        H: BuildHasher,
    {
        self.ring.get_node(point)
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
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client = Client::new();
    /// client.insert_node(&"node-1", 1);
    /// client.insert_point(&"point-1");
    /// ```
    pub fn insert_point(&mut self, point: &'a U)
    where
        U: Hash + Eq,
        H: BuildHasher,
    {
        let hash = util::gen_hash(&self.ring.hash_builder, point);
        if let Some((_, points)) = self.get_next_node(hash) {
            points.insert(point);
        } else {
            panic!("Error: empty ring.");
        }
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
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client = Client::new();
    /// client.insert_node(&"node-1", 1);
    /// client.insert_point(&"point-1");
    /// client.remove_point(&"point-1");
    /// ```
    pub fn remove_point(&mut self, point: &U)
    where
        U: Hash + Eq,
        H: BuildHasher,
    {
        let hash = util::gen_hash(&self.ring.hash_builder, &point);
        if let Some((_, points)) = self.get_next_node(hash) {
            points.remove(point);
        } else {
            panic!("Error: empty ring.");
        }
    }

    /// Returns the number of nodes in the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// client.insert_node(&"node-1", 3);
    /// assert_eq!(client.len(), 1);
    /// ```
    pub fn len(&self) -> usize
    where
        T: Hash + Eq,
    {
        self.ring.len()
    }

    /// Returns `true` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client: Client<&str, &str> = Client::new();
    ///
    /// assert!(client.is_empty());
    /// client.insert_node(&"node-1", 3);
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
    /// use hash_rings::consistent::Client;
    ///
    /// let mut client = Client::new();
    /// client.insert_node(&"node-1", 1);
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
        H: BuildHasher,
    {
        self.ring.iter().map(move |replica| {
            let mut points = Vec::new();
            for i in 0..replica.1 {
                let hash = util::combine_hash(
                    &self.ring.hash_builder,
                    util::gen_hash(&self.ring.hash_builder, &*replica.0),
                    util::gen_hash(&self.ring.hash_builder, &i),
                );
                points.extend(&self.data[&hash])
            }
            (replica.0, points)
        })
    }
}

impl<'a, T, U, H> IntoIterator for &'a Client<'a, T, U, H>
where
    T: Hash + Eq,
    U: Hash + Eq,
    H: BuildHasher,
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
    H: BuildHasher + Default,
{
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use crate::test_util::BuildDefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_size_empty() {
        let client: Client<'_, u32, u32> = Client::new();
        assert!(client.is_empty());
        assert_eq!(client.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_panic_remove_node_empty_client() {
        let mut client: Client<'_, u32, u32> = Client::new();
        client.insert_node(&0, 1);
        client.remove_node(&0);
    }

    #[test]
    #[should_panic]
    fn test_panic_remove_node_non_existent_node() {
        let mut client: Client<'_, u32, u32> = Client::new();
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

    pub struct Key(pub u32);
    impl Hash for Key {
        fn hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            state.write_u32(self.0 / 2);
        }
    }

    impl PartialEq for Key {
        fn eq(&self, other: &Key) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for Key {}

    #[test]
    fn test_insert_node_replace_node() {
        let mut client: Client<'_, Key, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&Key(0), 1);
        client.insert_point(&0);
        client.insert_node(&Key(1), 1);
        assert_eq!(client.get_points(&Key(1)).as_slice(), [&0u32]);
    }

    #[test]
    fn test_insert_node_share_node() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 1);
        client.insert_point(&0);
        client.insert_point(&3);
        client.insert_node(&1, 1);
        assert_eq!(client.get_points(&0).as_slice(), [&3u32]);
        assert_eq!(client.get_points(&1).as_slice(), [&0u32]);
    }

    #[test]
    fn test_remove_node() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 1);
        client.insert_point(&0);
        client.insert_node(&1, 1);
        client.remove_node(&1);
        assert_eq!(client.get_points(&0), [&0]);
    }

    #[test]
    fn test_get_node() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 3);
        assert_eq!(client.get_node(&0), &0);
    }

    #[test]
    fn test_insert_point() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 3);
        client.insert_point(&0);
        assert_eq!(client.get_points(&0).as_slice(), [&0u32]);
    }

    #[test]
    fn test_remove_point() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 3);
        client.insert_point(&0);
        client.remove_point(&0);
        let expected: [&u32; 0] = [];
        assert_eq!(client.get_points(&0).as_slice(), expected);
    }

    #[test]
    fn test_iter() {
        let mut client: Client<'_, u32, u32, BuildDefaultHasher> = Client::default();
        client.insert_node(&0, 3);
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
}
