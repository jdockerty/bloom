use std::{
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
};

use bit_vec::BitVec;

pub struct BloomFilter<K: Hash> {
    /// Internal bit vector representation.
    inner: BitVec,
    /// Number of bits the bit vector was initialised with.
    n_bits: usize,
    /// Number of times to run the specified hash
    k: usize,
    _phantom: PhantomData<K>,
}

impl<K: Hash + Debug> BloomFilter<K> {
    /// Create a new [`BloomFilter`].
    pub fn new(n_bits: usize, k: usize) -> Self {
        Self {
            inner: BitVec::from_elem(n_bits, false),
            n_bits,
            k,
            _phantom: PhantomData,
        }
    }

    /// Insert a value into the bloom filter.
    ///
    /// As this is a bloom filter, the value isn't _actually_ inserted. Only the
    /// hash of the item which was given. An internal bit vector is updated based
    /// on the hash of the contents that was provided.
    pub fn insert(&mut self, key: K) {
        let mut hasher = DefaultHasher::new();
        for _ in 0..self.k {
            key.hash(&mut hasher);
            let index = hasher.finish() as usize % self.n_bits;
            self.inner.set(index, true);
        }
    }
}

#[cfg(test)]
mod test {
    use bit_vec::BitVec;

    use super::BloomFilter;

    macro_rules! assert_bit_vec {
        ($bit_vec:expr, $n:literal, $($vals:literal),*) => {{
            // A false is used at the beginning here to start the expression.
            // The rest are logical ORs, so the `false` is simply to satisfy the
            // compiler checks
            assert_eq!($bit_vec, BitVec::from_fn($n, |i| { false $(|| i == $vals)* }));
        }};
    }

    #[test]
    fn create() {
        let bloom: BloomFilter<&str> = BloomFilter::new(10, 2);
        assert_eq!(bloom.n_bits, 10);
        assert_eq!(bloom.k, 2);
    }

    #[test]
    fn insertion() {
        let mut bloom: BloomFilter<&str> = BloomFilter::new(10, 2);
        bloom.insert("hello");
        // Results in 0100000100 for the internal bit vec
        assert_bit_vec!(bloom.inner, 10, 1, 7);
        bloom.insert("world");
        // Results in 0000100001 for the internal bit vec
        // Meaning that the overall vector is 0100100101
        assert_bit_vec!(bloom.inner, 10, 1, 7, 4, 9);
    }
}
