use std::marker::PhantomData;
use util;

/// A hashing ring implemented using jump hash.
///
/// Jump hash is based on using a hash of the key as the seed for a random number generator and
/// using it to jump forward in a list of nodes until it falls off the end. The last nodes it lands
/// in is the result.
///
/// Jump hash is very fast and executes in `O(ln n)` time. It also has no memory overhead and has
/// virtually perfect key distribution. However, the main limitation of jump hash is that it returns
/// an integer in the range [0, nodes) and it does not support arbitrary node names.
///
/// # Examples
/// ```
/// use hash_rings::jump::Ring;
///
/// let ring = Ring::new(100);
///
/// assert_eq!(ring.get_node("foo"), 1);
/// assert_eq!(ring.buckets(), 100);
/// ```
pub struct Ring<T>
where T: Hash
{
    nodes: u32,
    _marker: PhantomData<T>,
}

impl<T> Ring<T>
where T: Hash
{
    /// Constructs a new `Ring<T>` with a specified number of nodes.
    ///
    /// # Panics
    /// Panics if the number of nodes is zero.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::jump::Ring;
    ///
    /// let ring: Ring<u32> = Ring::new(100);
    /// ```
    pub fn new(nodes: u32) -> Self {
        assert!(nodes >= 1);
        Ring {
            nodes,
            _marker: PhantomData,
        }
    }

    /// Returns the node associated with a key.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::jump::Ring;
    ///
    /// let ring = Ring::new(100);
    /// assert_eq!(ring.get_node("foo"), 1);
    /// ```
    pub fn get_node(&self, key: &T) -> u32 {
        let h = util::gen_hash(key);
        let mut i: i64 = -1;
        let mut j: i64 = 0;

        while j < (self.nodes as i64) {
            i = j;
            h = h.wrapping_mul(2862933555777941757).wrapping_add(1);
            j = (((i.wrapping_add(1)) as f64) * ((1i64 << 31) as f64) / (((h >> 33).wrapping_add(1)) as f64)) as i64;
        }
        b as u32
    }

    /// Returns the number of nodes in the ring.
    ///
    /// # Examples
    /// ```
    /// use hash_rings::jump::Ring;
    ///
    /// let ring = Ring::new(100);
    /// assert_eq!(ring.buckets(), 100);
    /// ```
    pub fn nodes(&self) -> u32 {
        self.nodes
    }
}

#[cfg(test)]
mod tests {
    use super::Ring;

    #[test]
    fn test_get_node() {
        let ring = Ring::new(100);
        assert_eq!(ring.get_node("foo"), 1);
    }

    #[test]
    fn test_nodes() {
        let ring: u32 = Ring::new(100);
        assert_eq!(ring.nodes(), 100);
    }
}
