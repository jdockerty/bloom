use std::hash::Hasher;

use bit_vec::BitVec;

pub struct BloomFilter<H: Hasher> {
    inner: BitVec,
    n_bits: usize,
    hasher: H,
    /// Number of times to run the specified hash
    k: usize,
}

impl<H: Hasher> BloomFilter<H> {
    pub fn new(n_bits: usize, k: usize, hasher: H) -> Self {
        Self {
            inner: BitVec::from_elem(n_bits, false),
            n_bits,
            hasher,
            k,
        }
    }
}

#[cfg(test)]
mod test {
    use std::hash::DefaultHasher;

    use super::BloomFilter;

    #[test]
    fn create_bloom_filter() {
        let bloom = BloomFilter::new(100, 2, DefaultHasher::new());
        assert_eq!(bloom.n_bits, 100);
        assert_eq!(bloom.k, 2);
    }
}
