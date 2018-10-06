//! Hashing ring implemented using multi-probe consistent hashing.

use rand::{Rng, XorShiftRng};
use siphasher::sip::SipHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use util;

const PRIME: u64 = 0xFFFF_FFFF_FFFF_FFC5;

/// A hashing ring implemented using multi-probe consistent hashing.
///
/// Multi-probe consistent hashing is a variation on consistent hashing where instead of the nodes
/// being hashed multiple times to reduce variance, the keys are hashed multiple times. Each key is
/// hashed `hash_count` times and the closest node over all hashes is returned.
///
/// # Examples
/// ```
/// use hash_rings::mpc::Ring;
///
/// let mut ring = Ring::new(2);
///
/// ring.insert_node(&"node-1");
/// ring.insert_node(&"node-2");
///
/// ring.remove_node(&"node-1");
///
/// assert_eq!(ring.get_node(&"point-1"), &"node-2");
/// assert_eq!(ring.len(), 1);
///
/// let mut iterator = ring.iter();
/// assert_eq!(iterator.next(), Some(&"node-2"));
/// assert_eq!(iterator.next(), None);
/// ```
pub struct Ring<'a, T>
where
    T: 'a,
{
    nodes: BTreeMap<u64, &'a T>,
    hash_count: u64,
    hashers: [SipHasher; 2],
}

impl<'a, T> Ring<'a, T>
where
    T: Hash + Eq,
{
    fn get_hashers() -> [SipHasher; 2] {
        let mut rng = XorShiftRng::new_unseeded();
        [
            SipHasher::new_with_keys(rng.next_u64(), rng.next_u64()),
            SipHasher::new_with_keys(rng.next_u64(), rng.next_u64()),
        ]
    }

    fn get_hashes<U>(&self, item: &U) -> [u64; 2]
    where
        U: Hash,
    {
        let mut ret = [0; 2];
        for (index, hash) in ret.iter_mut().enumerate() {
            let mut sip = self.hashers[index];
            item.hash(&mut sip);
            *hash = sip.finish();
        }
        ret
    }

    fn get_distance(hash: u64, next_hash: u64) -> u64 {
        if hash > next_hash {
            next_hash + (<u64>::max_value() - hash)
        } else {
            next_hash - hash
        }
    }

    /// Constructs a new, empty `Ring<T>` that hashes `hash_count` times when a key is inserted.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::mpc::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new(2);
    /// ```
    pub fn new(hash_count: u64) -> Self {
        assert!(hash_count > 0);
        Ring {
            nodes: BTreeMap::new(),
            hash_count,
            hashers: Self::get_hashers(),
        }
    }

    fn get_next_hash(&self, hash: u64) -> u64 {
        let next_hash_opt = self
            .nodes
            .range(hash..)
            .next()
            .or_else(|| self.nodes.iter().next())
            .map(|entry| *entry.0);
        match next_hash_opt {
            Some(hash) => hash,
            None => panic!("Error: empty ring."),
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
    /// use hash_rings::mpc::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new(2);
    /// ring.insert_node(&"node-1");
    /// ```
    pub fn insert_node(&mut self, id: &'a T) {
        self.nodes.insert(util::gen_hash(id), id);
    }

    /// Removes a node.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::mpc::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new(2);
    ///
    /// ring.insert_node(&"node-1");
    /// ring.remove_node(&"node-1");
    /// ```
    pub fn remove_node(&mut self, id: &T) {
        self.nodes.remove(&util::gen_hash(id));
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
    /// use hash_rings::mpc::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new(2);
    ///
    /// ring.insert_node(&"node-1");
    /// assert_eq!(ring.get_node(&"point-1"), &"node-1");
    /// ```
    pub fn get_node<U>(&self, point: &U) -> &T
    where
        U: Hash,
    {
        let hashes = self.get_hashes(point);
        let hash = (0..self.hash_count)
            .map(|i| {
                let hash = hashes[0].wrapping_add((i as u64).wrapping_mul(hashes[1]) % PRIME);
                let next_hash = self.get_next_hash(hash);
                (Self::get_distance(hash, next_hash), next_hash)
            })
            .min()
            .expect("Error: expected positive hash count.");

        self.nodes[&hash.1]
    }

    /// Returns the number of nodes in the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::mpc::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new(2);
    ///
    /// ring.insert_node(&"node-1");
    /// assert_eq!(ring.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::mpc::Ring;
    ///
    /// let mut ring: Ring<&str> = Ring::new(2);
    ///
    /// assert!(ring.is_empty());
    /// ring.insert_node(&"node-1");
    /// assert!(!ring.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns an iterator over the ring. The iterator will yield the nodes in the ring in no
    /// particular order.
    ///
    /// # Examples
    ///
    /// ```
    /// use hash_rings::mpc::Ring;
    ///
    /// let mut ring = Ring::new(2);
    /// ring.insert_node(&"node-1");
    ///
    /// let mut iterator = ring.iter();
    /// assert_eq!(iterator.next(), Some(&"node-1"));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&'a self) -> impl Iterator<Item = &'a T> {
        self.nodes.iter().map(|node| *node.1)
    }
}

impl<'a, T> IntoIterator for &'a Ring<'a, T>
where
    T: Hash + Eq,
{
    type IntoIter = Box<Iterator<Item = &'a T> + 'a>;
    type Item = (&'a T);

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::Ring;

    #[test]
    #[should_panic]
    fn test_new_zero_hash_count() {
        let _ring: Ring<u32> = Ring::new(0);
    }

    #[test]
    #[should_panic]
    fn test_get_node_empty_ring() {
        let ring: Ring<u32> = Ring::new(2);
        ring.get_node(&0);
    }

    #[test]
    fn test_get_node() {
        let mut ring = Ring::new(2);

        ring.insert_node(&0);
        assert_eq!(ring.get_node(&2), &0);

        ring.insert_node(&1);
        assert_eq!(ring.get_node(&2), &1);

        ring.remove_node(&1);
        assert_eq!(ring.get_node(&2), &0);
    }

    #[test]
    fn test_len() {
        let mut ring = Ring::new(2);
        ring.insert_node(&0);

        assert_eq!(ring.len(), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut ring = Ring::new(2);
        assert!(ring.is_empty());

        ring.insert_node(&0);
        assert!(!ring.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut ring = Ring::new(2);
        ring.insert_node(&0);

        let mut iterator = ring.iter();
        assert_eq!(iterator.next(), Some(&0));
        assert_eq!(iterator.next(), None);
    }
}
