use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use bit_vec::BitVec;
use fxhash::FxHasher;

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
        let mut h = FxHasher::default();
        for _ in 0..self.k {
            let index = self.hash_index(&key, &mut h);
            self.inner.set(index, true);
        }
    }

    /// Get the hash index to set the bit as occupied within the internal bit
    /// vector. This automatically applies the modulo of the number of bits
    /// within the bit vector and is therefore ready to use.
    fn hash_index<H: Hasher>(&mut self, key: &K, hasher: &mut H) -> usize {
        key.hash(hasher);
        hasher.finish() as usize % self.n_bits
    }

    /// Determine whether a value is contained within the bloom filter.
    ///
    /// # Notes
    ///
    /// This can return false positives, but can not return a false negative.
    ///
    /// In other words this can return `true` for a value which is not in the
    /// set because another item happened to flip the bits within the vector
    /// as it resulted in the same hash index.
    /// However, when any of the bits are 0 for an item this means the value is
    /// definitely not within the set and we can return `false` for certain.
    pub fn check(&mut self, key: K) -> bool {
        let mut h = FxHasher::default();
        for _ in 0..self.k {
            let index = self.hash_index(&key, &mut h);
            match self
                .inner
                .get(index)
                .expect("Modulo ensures that this is always in-bounds")
            {
                true => continue,
                // We can instantly return here as if the value has passed
                // through the filter before then the bit would have definitely
                // been set to `true` already.
                false => return false,
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use bit_vec::BitVec;

    use super::BloomFilter;

    /// Assertion over a provided [`BitVec`] and one which is constructed based
    /// on provided integer literals to the [`create_bit_vec`] macro.
    macro_rules! assert_bit_vec {
        ($bit_vec:expr, $($vals:literal),*) => {
            assert_eq!($bit_vec, create_bit_vec!($bit_vec.len(), $($vals),*));
        };
    }

    /// Create a [`BitVec`] with the provided indexes being already flipped to
    /// true (1).
    macro_rules! create_bit_vec {
        ($size:expr, $($indexes:literal),*) => {
            // A false is used at the beginning here to start the expression.
            // The rest are logical ORs, so the `false` is simply to satisfy the
            // compiler checks
            BitVec::from_fn($size, |i| { false $(|| i == $indexes)*})
        };
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
        assert_bit_vec!(bloom.inner, 0, 6);
        bloom.insert("world");
        assert_bit_vec!(bloom.inner, 0, 2, 4, 6);
    }

    #[test]
    fn check() {
        let mut bloom: BloomFilter<&str> = BloomFilter::new(10, 2);
        bloom.insert("hello");
        assert!(bloom.check("hello"));
        assert!(!bloom.check("world"));
    }
}
