use primal::Sieve;
use rand::{Rng, XorShiftRng};
use siphasher::sip::SipHasher;
use std::hash::{Hash, Hasher};
use std::iter;

/// A hashing ring implemented using maglev hashing.
///
/// Maglev hashing produces a lookup table that allows finding a node in constant time by
/// generating random permutations.
///
/// # Examples
/// ```
/// use hash_rings::maglev::Ring;
///
/// let ring = Ring::new(vec![&"node-1", &"node-2", &"node-3"]);
///
/// assert_eq!(ring.get_node(&"point-1"), &"node-3");
/// assert_eq!(ring.nodes(), 3);
/// assert_eq!(ring.capacity(), 307);
/// ```
pub struct Ring<'a, T>
where
    T: 'a + Hash,
{
    nodes: Vec<&'a T>,
    lookup: Vec<usize>,
    hasher: SipHasher,
}

impl<'a, T> Ring<'a, T>
where
    T: 'a + Hash,
{
    fn get_hashers() -> [SipHasher; 2] {
        let mut rng = XorShiftRng::new_unseeded();
        [
            SipHasher::new_with_keys(rng.next_u64(), rng.next_u64()),
            SipHasher::new_with_keys(rng.next_u64(), rng.next_u64()),
        ]
    }

    /// Constructs a new `Ring<T>` with a specified list of nodes.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::maglev::Ring;
    ///
    /// let ring = Ring::new(vec![&"node-1", &"node-2", &"node-3"]);
    /// ```
    pub fn new(nodes: Vec<&'a T>) -> Self {
        assert!(!nodes.is_empty());
        let capacity_hint = nodes.len() * 100;
        Ring::with_capacity_hint(nodes, capacity_hint)
    }

    /// Constructs a new `Ring<T>` with a specified list of nodes and a capacity hint. The actual
    /// capacity of the ring will always be the next prime greater than or equal to
    /// `capacity_hint`. If nodes are removed and the ring is regenerated, the ring should be
    /// rebuilt with the same capacity.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::maglev::Ring;
    ///
    /// let ring = Ring::with_capacity_hint(vec![&"node-1", &"node-2", &"node-3"], 100);
    /// assert_eq!(ring.capacity(), 101);
    /// ```
    pub fn with_capacity_hint(nodes: Vec<&'a T>, capacity_hint: usize) -> Self {
        let hashers = Self::get_hashers();
        let lookup = Self::populate(&hashers, &nodes, capacity_hint);
        Ring {
            nodes,
            lookup,
            hasher: hashers[0],
        }
    }

    fn get_hash<U>(hasher: SipHasher, key: &U) -> usize
    where
        U: Hash,
    {
        let mut sip = hasher;
        key.hash(&mut sip);
        sip.finish() as usize
    }

    fn populate(hashers: &[SipHasher; 2], nodes: &[&T], capacity_hint: usize) -> Vec<usize> {
        let m = Sieve::new(capacity_hint * 2)
            .primes_from(capacity_hint)
            .next()
            .unwrap();
        let n = nodes.len();

        let permutation: Vec<Vec<usize>> = nodes
            .iter()
            .map(|node| {
                let offset = Self::get_hash(hashers[0], node) % m;
                let skip = (Self::get_hash(hashers[1], node) % (m - 1)) + 1;
                (0..m).map(|i| (offset + i * skip) % m).collect()
            })
            .collect();

        let mut next: Vec<usize> = iter::repeat(0).take(n).collect();
        let mut entry: Vec<usize> = iter::repeat(<usize>::max_value()).take(m).collect();

        let mut i = 0;
        while i < m {
            for j in 0..n {
                let mut c = permutation[j][next[j]];
                while entry[c] != <usize>::max_value() {
                    next[j] += 1;
                    c = permutation[j][next[j]];
                }
                entry[c] = j;
                next[j] += 1;
                i += 1;

                if i == m {
                    break;
                }
            }
        }

        entry
    }

    /// Returns the number of nodes in the ring.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::maglev::Ring;
    ///
    /// let ring = Ring::new(vec![&"node-1", &"node-2", &"node-3"]);
    /// assert_eq!(ring.nodes(), 3);
    /// ```
    pub fn nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the capacity of the ring. If nodes are removed and the ring is regenerated, the
    /// ring should be rebuilt with the same capacity.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::maglev::Ring;
    ///
    /// let ring = Ring::new(vec![&"node-1", &"node-2", &"node-3"]);
    /// assert_eq!(ring.capacity(), 307);
    /// ```
    pub fn capacity(&self) -> usize {
        self.lookup.len()
    }

    /// Returns the node associated with a key.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::maglev::Ring;
    ///
    /// let ring = Ring::new(vec![&"node-1", &"node-2", &"node-3"]);
    /// assert_eq!(ring.get_node(&"point-1"), &"node-3");
    /// ```
    pub fn get_node<U>(&self, key: &U) -> &T
    where
        U: Hash,
    {
        let index = Self::get_hash(self.hasher, key) % self.capacity();
        self.nodes[self.lookup[index]]
    }

    /// Returns an iterator over the ring. The iterator will yield the nodes in the ring.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::maglev::Ring;
    ///
    /// let ring = Ring::new(vec![&"node-1", &"node-2", &"node-3"]);
    ///
    /// let mut iterator = ring.iter();
    /// assert_eq!(iterator.next(), Some(&"node-1"));
    /// assert_eq!(iterator.next(), Some(&"node-2"));
    /// assert_eq!(iterator.next(), Some(&"node-3"));
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&'a self) -> Box<Iterator<Item = &'a T> + 'a> {
        Box::new(self.nodes.iter().map(|node| *node))
    }
}

impl<'a, T> IntoIterator for &'a Ring<'a, T>
where
    T: Hash + Eq,
{
    type Item = (&'a T);
    type IntoIter = Box<Iterator<Item = &'a T> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::Ring;

    #[test]
    #[should_panic]
    fn test_new_empty() {
        let _ring: Ring<u32> = Ring::new(vec![]);
    }

    #[test]
    fn test_get_node() {
        let ring = Ring::new(vec![&0, &1, &2]);
        assert_eq!(ring.get_node(&0), &0);
        assert_eq!(ring.get_node(&1), &1);

        let ring = Ring::with_capacity_hint(vec![&0, &1], ring.capacity());
        assert_eq!(ring.get_node(&0), &0);
        assert_eq!(ring.get_node(&1), &1);
    }

    #[test]
    fn test_nodes() {
        let ring = Ring::new(vec![&0, &1, &2]);
        assert_eq!(ring.nodes(), 3);
    }

    #[test]
    fn test_capacity() {
        let ring = Ring::new(vec![&0, &1, &2]);
        assert_eq!(ring.capacity(), 307);

        let ring = Ring::with_capacity_hint(vec![&0, &1], ring.capacity());
        assert_eq!(ring.capacity(), 307);
    }

    #[test]
    fn test_iter() {
        let ring = Ring::new(vec![&0, &1, &2]);

        let mut iterator = ring.iter();
        assert_eq!(iterator.next(), Some(&0));
        assert_eq!(iterator.next(), Some(&1));
        assert_eq!(iterator.next(), Some(&2));
        assert_eq!(iterator.next(), None);
    }
}
