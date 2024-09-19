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
    hasher: DefaultHasher,
}

impl<K: Hash + Debug> BloomFilter<K> {
    /// Create a new [`BloomFilter`].
    pub fn new(n_bits: usize, k: usize) -> Self {
        Self {
            inner: BitVec::from_elem(n_bits, false),
            n_bits,
            k,
            _phantom: PhantomData,
            hasher: DefaultHasher::new(),
        }
    }

    /// Insert a value into the bloom filter.
    ///
    /// As this is a bloom filter, the value isn't _actually_ inserted. Only the
    /// hash of the item which was given. An internal bit vector is updated based
    /// on the hash of the contents that was provided.
    pub fn insert(&mut self, key: K) {
        for _ in 0..self.k {
            key.hash(&mut self.hasher);
            let index = self.hash_index(&key);
            println!("Insert {key:?}={index}");
            self.inner.set(index, true);
        }
    }

    fn hash_index(&mut self, key: &K) -> usize {
        key.hash(&mut self.hasher);
        self.hasher.finish() as usize % self.n_bits
    }

    pub fn get(&mut self, key: K) -> bool {
        let mut exists = Vec::with_capacity(self.k);
        for _ in 0..self.k {
            let index = self.hash_index(&key);
            println!("Get {key:?}={index}");
            exists.push(
                self.inner
                    .get(index)
                    .expect("Modulo ensures that this is always in-bounds"),
            );
        }
        println!("{exists:?}");
        exists.iter().all(|&i| i)
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
        assert_bit_vec!(bloom.inner, 1, 7);
        bloom.insert("world");
        assert_bit_vec!(bloom.inner, 1, 7, 0, 3);
    }

    #[test]
    fn get() {
        let mut bloom: BloomFilter<&str> = BloomFilter::new(10, 2);
        bloom.insert("hello");
        assert_eq!(bloom.get("hello"), true);
        assert_eq!(bloom.get("world"), false);
    }
}
