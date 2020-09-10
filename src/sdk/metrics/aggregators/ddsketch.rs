//! DDSketch quantile sketch with relative-error guarantees.
//! DDSketch is a fast and fully-mergeable quantile sketch with relative-error guarantees.
//!
//! The detail of this algorithm can be found in https://arxiv.org/pdf/1908.10693
//!
//! The main difference between this approach and previous art is DDSKetch employ a new method to
//! compute the error. Traditionally, the error rate of one sketch is evaluated by rank accuracy,
//! which can still generate a relative large variance if the dataset has long tail.
//!
//! DDSKetch, on the contrary, employs relative error rate that could work well on long tail dataset.
//!

use std::ops::AddAssign;
use std::sync::{Arc, RwLock};

use crate::{
    api::metrics::{Descriptor, MetricsError, Number, NumberKind, Result},
    sdk::export::metrics::{
        Aggregator, Count, Distribution, Max, Min, MinMaxSumCount, Quantile, Sum,
    },
};
use std::any::Any;
use std::cmp::Ordering;
use std::mem;

const INITIAL_NUM_BINS: usize = 128;
const GROW_LEFT_BY: i64 = 128;

const DEFAULT_MAX_NUM_BINS: i64 = 2048;
const DEFAULT_ALPHA: f64 = 0.01;
const DEFAULT_MIN_BOUNDARY: f64 = 1.0e-9;

/// DDSKetch quantile sketch algorithm
///
/// It can give q-quantiles with α-accurate for any 0<=q<=1
#[derive(Debug)]
pub struct DDSKetchAggregator {
    inner: RwLock<Inner>,
}

impl DDSKetchAggregator {
    /// Create a new DDSKetchAggregator.
    pub fn new(
        alpha: f64,
        max_num_bins: i64,
        min_boundary: f64,
        kind: NumberKind,
    ) -> DDSKetchAggregator {
        DDSKetchAggregator {
            inner: RwLock::new(Inner::new(alpha, max_num_bins, min_boundary, kind)),
        }
    }
}

impl Default for DDSKetchAggregator {
    fn default() -> Self {
        DDSKetchAggregator::new(
            DEFAULT_ALPHA,
            DEFAULT_MAX_NUM_BINS,
            DEFAULT_MIN_BOUNDARY,
            NumberKind::F64,
        )
    }
}

impl Sum for DDSKetchAggregator {
    fn sum(&self) -> Result<Number> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| inner.sum.clone())
    }
}

impl Min for DDSKetchAggregator {
    fn min(&self) -> Result<Number> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| inner.min_value.clone())
    }
}

impl Max for DDSKetchAggregator {
    fn max(&self) -> Result<Number> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| inner.max_value.clone())
    }
}

impl Count for DDSKetchAggregator {
    fn count(&self) -> Result<u64> {
        self.inner
            .read()
            .map_err(From::from)
            .map(|inner| inner.store.count)
    }
}

impl MinMaxSumCount for DDSKetchAggregator {}

impl Distribution for DDSKetchAggregator {}

impl Quantile for DDSKetchAggregator {
    fn quantile(&self, q: f64) -> Result<Number> {
        if q < 0.0 || q > 1.0 {
            return Err(MetricsError::InvalidQuantile);
        }
        self.inner.read().map_err(From::from).and_then(|inner| {
            if inner.store.count == 0 {
                return Err(MetricsError::NoDataCollected);
            }
            if q == 0.0 {
                return Ok(inner.min_value.clone());
            }

            if (q - 1.0).abs() < f64::EPSILON {
                return Ok(inner.max_value.clone());
            }

            let rank = (q * (inner.store.count - 1) as f64).ceil() as u64 + 1;
            let mut key = inner.store.key_at_rank(rank);
            // Calculate the actual value based on the key of bins.
            let quantile_val = match key.cmp(&0) {
                Ordering::Less => {
                    key += inner.offset;
                    -2.0 * inner.gamma_ln * (-key as f64) / (1.0 + inner.gamma)
                }
                Ordering::Greater => {
                    key -= inner.offset;
                    2.0 * inner.gamma_ln * (key as f64) / (1.0 + inner.gamma)
                }
                Ordering::Equal => 0f64,
            };

            let mut quantile = match inner.kind {
                NumberKind::F64 => Number::from(quantile_val),
                NumberKind::U64 => Number::from(quantile_val as u64),
                NumberKind::I64 => Number::from(quantile_val as i64),
            };

            // Make sure the result of quantile is within [min_value, max_value]
            if quantile.partial_cmp(&inner.kind, &inner.min_value) == Some(Ordering::Less) {
                quantile = inner.min_value.clone();
            }

            if quantile.partial_cmp(&inner.kind, &inner.max_value) == Some(Ordering::Greater) {
                quantile = inner.max_value.clone();
            }

            Ok(quantile)
        })
    }
}

impl Aggregator for DDSKetchAggregator {
    fn update(&self, number: &Number, descriptor: &Descriptor) -> Result<()> {
        self.inner
            .write()
            .map_err(From::from)
            .map(|mut inner| inner.add(number, descriptor.number_kind()))
    }

    fn synchronized_move(
        &self,
        destination: &Arc<(dyn Aggregator + Send + Sync)>,
        descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other) = destination.as_any().downcast_ref::<Self>() {
            other
                .inner
                .write()
                .map_err(From::from)
                .and_then(|mut other| {
                    self.inner.write().map_err(From::from).map(|mut inner| {
                        let kind = descriptor.number_kind();
                        other.max_value = mem::replace(&mut inner.max_value, kind.zero());
                        other.min_value = mem::replace(&mut inner.min_value, kind.zero());
                        other.min_boundary = mem::take(&mut inner.min_boundary);
                        other.offset = mem::take(&mut inner.offset);
                        other.gamma = mem::take(&mut inner.gamma);
                        other.gamma_ln = mem::take(&mut inner.gamma_ln);
                        other.store = mem::take(&mut inner.store);
                        other.sum = mem::replace(&mut inner.sum, kind.zero());
                    })
                })
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, destination
            )))
        }
    }

    fn merge(
        &self,
        other: &(dyn Aggregator + Send + Sync),
        _descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<DDSKetchAggregator>() {
            self.inner.write()
                .map_err(From::from)
                .and_then(|mut inner| {
                    other.inner.read()
                        .map_err(From::from)
                        .and_then(|other| {
// assert that it can merge
                            if inner.store.max_num_bins != other.store.max_num_bins {
                                return Err(MetricsError::InconsistentAggregator(format!(
                                    "When merging two DDSKetchAggregators, their max number of bins must be the same. Expect max number of bins to be {:?}, but get {:?}", inner.store.max_num_bins, other.store.max_num_bins
                                )));
                            }
                            if (inner.gamma - other.gamma).abs() > f64::EPSILON {
                                return Err(MetricsError::InconsistentAggregator(format!(
                                    "When merging two DDSKetchAggregators, their gamma must be the same. Expect max number of bins to be {:?}, but get {:?}", inner.gamma, other.gamma
                                )));
                            }

                            if other.store.count == 0 {
                                return Ok(());
                            }

                            if inner.store.count == 0 {
                                inner.store.merge(&other.store);
                                inner.sum = other.sum.clone();
                                inner.min_value = other.min_value.clone();
                                inner.max_value = other.max_value.clone();
                            }

                            inner.store.merge(&other.store);

                            inner.sum = match inner.kind {
                                NumberKind::F64 =>
                                    Number::from(inner.sum.to_f64(&inner.kind) + other.sum.to_f64(&other.kind)),
                                NumberKind::U64 => Number::from(inner.sum.to_u64(&inner.kind) + other.sum.to_u64(&other.kind)),
                                NumberKind::I64 => Number::from(inner.sum.to_i64(&inner.kind) + other.sum.to_i64(&other.kind))
                            };

                            if inner.min_value.partial_cmp(&inner.kind, &other.min_value) == Some(Ordering::Greater) {
                                inner.min_value = other.min_value.clone();
                            };

                            if inner.max_value.partial_cmp(&inner.kind, &other.max_value) == Some(Ordering::Less) {
                                inner.max_value = other.max_value.clone();
                            }

                            Ok(())
                        })
                })
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, other
            )))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// DDSKetch implementation.
/// Note that Inner is not thread-safe. All operation should be protected by a lock or other
/// synchronization.
///
/// Inner will also convert all Number into actual primitive type and back.
#[derive(Debug)]
struct Inner {
    store: Store,
    kind: NumberKind,
    // sum of all value within store
    sum: Number,
    // γ = (1 + α)/(1 - α)
    gamma: f64,
    // ln(γ)
    gamma_ln: f64,
    // The minimum possible value that could be stored in DDSKetch
    min_boundary: f64,
    // offset is here to ensure that keys for positive numbers that are larger than min_value are
    // greater than or equal to 1 while the keys for negative numbers are less than or equal to -1.
    offset: i64,

    // minimum number that in store.
    min_value: Number,
    // maximum number that in store.
    max_value: Number,
}

impl Inner {
    fn new(alpha: f64, max_num_bins: i64, min_boundary: f64, kind: NumberKind) -> Inner {
        let gamma: f64 = 1.0 + 2.0 * alpha / (1.0 - alpha);
        let mut inner = Inner {
            store: Store::new(max_num_bins),
            min_value: kind.max(),
            max_value: kind.min(),
            sum: kind.zero(),
            gamma,
            gamma_ln: gamma.ln(),
            min_boundary,
            offset: 0,
            kind,
        };
        // reset offset based on min_value
        inner.offset = -(inner.log_gamma(inner.min_boundary)).ceil() as i64 + 1i64;
        inner
    }

    fn add(&mut self, v: &Number, kind: &NumberKind) {
        let key = self.key(v, kind);
        self.store.add(key);

        // update min and max
        if self.min_value.partial_cmp(&self.kind, v) == Some(Ordering::Greater) {
            self.min_value = v.clone();
        }

        if self.max_value.partial_cmp(&self.kind, v) == Some(Ordering::Less) {
            self.max_value = v.clone();
        }

        match &self.kind {
            NumberKind::I64 => {
                self.sum = Number::from(self.sum.to_i64(&self.kind) + v.to_i64(kind));
            }
            NumberKind::U64 => {
                self.sum = Number::from(self.sum.to_u64(&self.kind) + v.to_u64(kind));
            }
            NumberKind::F64 => {
                self.sum = Number::from(self.sum.to_f64(&self.kind) + v.to_f64(kind));
            }
        }
    }

    fn key(&self, num: &Number, kind: &NumberKind) -> i64 {
        if num.to_f64(kind) < -self.min_boundary {
            let positive_num = match kind {
                NumberKind::F64 => Number::from(-num.to_f64(kind)),
                NumberKind::U64 => Number::from(num.to_u64(kind)),
                NumberKind::I64 => Number::from(-num.to_i64(kind)),
            };
            (-self.log_gamma(positive_num.to_f64(kind)).ceil()) as i64 - self.offset
        } else if num.to_f64(kind) > self.min_boundary {
            self.log_gamma(num.to_f64(&kind)).ceil() as i64 + self.offset
        } else {
            0i64
        }
    }

    /// get the index of the bucket based on num
    fn log_gamma(&self, num: f64) -> f64 {
       num.ln() / self.gamma_ln
    }
}

#[derive(Debug)]
struct Store {
    bins: Vec<u64>,
    count: u64,
    min_key: i64,
    max_key: i64,
    // maximum number of bins Store can have.
    // In the worst case, the bucket can grow as large as the number of the elements inserted into.
    // max_num_bins helps control the number of bins.
    max_num_bins: i64,
}

impl Default for Store {
    fn default() -> Self {
        Store {
            bins: vec![0; INITIAL_NUM_BINS],
            count: 0,
            min_key: 0,
            max_key: 0,
            max_num_bins: DEFAULT_MAX_NUM_BINS,
        }
    }
}

/// DDSKetchInner stores the data
impl Store {
    fn new(max_num_bins: i64) -> Store {
        Store {
            bins: vec![0; INITIAL_NUM_BINS],
            count: 0u64,
            min_key: 0i64,
            max_key: 0i64,
            max_num_bins,
        }
    }

    /// Add count based on key.
    ///
    /// If key is not in [min_key, max_key], we will expand to left or right
    ///
    ///
    /// The bins are essentially working in a round-robin fashion where we can use all space in bins
    /// to represent any continuous space within length. That's why we need to offset the key
    /// with `min_key` before add 1 in bins.
    fn add(&mut self, key: i64) {
        if self.count == 0 {
            self.max_key = key;
            self.min_key = key - self.bins.len() as i64 + 1
        }

        if key < self.min_key {
            self.grow_left(key)
        } else if key > self.max_key {
            self.grow_right(key)
        }
        let idx = if key - self.min_key < 0 {
            0
        } else {
            key - self.min_key
        };
        // we unwrap here because grow_left or grow_right will make sure the idx is less than vector size
        let bin_count = self.bins.get_mut(idx as usize).unwrap();
        *bin_count += 1;
        self.count += 1;
    }

    fn grow_left(&mut self, key: i64) {
        if self.min_key < key || self.bins.len() >= self.max_num_bins as usize {
            return;
        }

        let min_key = if self.max_key - key >= self.max_num_bins {
            self.max_key - self.max_num_bins + 1
        } else {
            let mut min_key = self.min_key;
            while min_key > key {
                min_key -= GROW_LEFT_BY;
            }
            min_key
        };

        // The new vector will contain three parts.
        // First part is all 0, which is the part expended
        // Second part is from existing bins.
        // Third part is what's left.
        let expected_len = (self.max_key - min_key + 1) as usize;
        let mut new_bins = vec![0u64; expected_len];
        let old_bin_slice = &mut new_bins[(self.min_key - min_key) as usize..];
        old_bin_slice.copy_from_slice(&self.bins);

        self.bins = new_bins;
        self.min_key = min_key;
    }

    fn grow_right(&mut self, key: i64) {
        if self.max_key > key {
            return;
        }

        // Adjust max key
        if key - self.max_key >= self.max_num_bins {
            self.bins = vec![0; self.max_num_bins as usize];
            self.max_key = key;
            self.min_key = key - self.max_num_bins + 1;
            self.bins.get_mut(0).unwrap().add_assign(self.count);
        } else if key - self.min_key >= self.max_num_bins {
            let min_key = key - self.max_num_bins + 1;
            let upper_bound = if min_key < self.max_key + 1 {
                min_key
            } else {
                self.max_key + 1
            } - self.min_key;
            let n = self.bins.iter().take(upper_bound as usize).sum::<u64>();

            if self.bins.len() < self.max_num_bins as usize {
                let mut new_bins = vec![0; self.max_num_bins as usize];
                new_bins.copy_from_slice(&self.bins[(min_key - self.min_key) as usize..]);
                self.bins = new_bins;
            } else {
                // bins length is equal to max number of bins
                self.bins.drain(0..(min_key - self.min_key) as usize);

                for _ in self.max_key - min_key + 1..self.max_num_bins {
                    self.bins.push(0);
                }
            }
            self.max_key = key;
            self.min_key = min_key;
            self.bins.get_mut(0).unwrap().add_assign(n);
        } else {
            let mut new_bin = vec![0; (key - self.min_key + 1) as usize];
            new_bin[0..self.bins.len()]
                .as_mut()
                .copy_from_slice(&self.bins);
            self.bins = new_bin;
            self.max_key = key;
        }
    }

    /// Returns the key of values at rank
    fn key_at_rank(&self, rank: u64) -> i64 {
        self.bins
            .iter()
            .enumerate()
            .scan(0, |state, (key, &count)| {
                *state += count;
                Some((key, *state))
            })
            .filter(|(_key, accumulated)| *accumulated >= rank)
            .map(|(key, _)| key as i64 + self.min_key)
            .next()
            .unwrap_or(self.max_key)
    }

    /// Merge two stores
    fn merge(&mut self, other: &Store) {
        if self.count == 0 {
            return;
        }
        if other.count == 0 {
            self.bins = other.bins.clone();
            self.min_key = other.min_key;
            self.max_key = other.max_key;
            self.count = other.count;
        }

        if self.max_key > other.max_key {
            if other.min_key < self.min_key {
                self.grow_left(other.min_key);
            }
            let start = if other.min_key > self.min_key {
                other.min_key
            } else {
                self.min_key
            } as usize;
            for i in start..other.max_key as usize {
                self.bins[i - self.min_key as usize] = other.bins[i - other.min_key as usize];
            }
            let mut n = 0;
            for i in other.min_key as usize..self.min_key as usize {
                n += other.bins[i - other.min_key as usize]
            }
            self.bins[0] += n;
        } else if other.min_key < self.min_key {
            let mut tmp_bins = vec![0u64; other.bins.len()];
            tmp_bins.as_mut_slice().copy_from_slice(&other.bins);

            for i in self.min_key as usize..self.max_key as usize {
                tmp_bins[i - other.min_key as usize] += self.bins[i - self.min_key as usize];
            }

            self.bins = tmp_bins;
            self.max_key = other.max_key;
            self.min_key = other.min_key;
        } else {
            self.grow_right(other.max_key);
            for i in other.min_key as usize..(other.max_key + 1) as usize {
                self.bins[i - self.min_key as usize] += other.bins[i - other.min_key as usize];
            }
        }

        self.count += other.count;
    }
}

#[cfg(test)]
mod tests {
    use crate::sdk::metrics::aggregators::ddsketch::Store;

    /// First set max_num_bins < number of keys, test to see if the store will collapse to left
    /// most bin instead of expending beyond the max_num_bins
    #[test]
    fn test_insert_into_store() {
        let mut store = Store::new(200);
        for i in 0..1400 {
            store.add(i)
        }
        assert_eq!(store.count, 1400);
        assert_eq!(store.bins.len(), 200 as usize);
    }

    /// Before merge, store1 should hold 300 bins that looks like [201,1,1,1,...],
    /// store 2 should hold 200 bins looks like [301,1,1,...]
    /// After merge, store 1 should still hold 300 bins with following distribution
    ///
    /// index [0,0] -> 201
    ///
    /// index [1,99] -> 1
    ///
    /// index [100, 100] -> 302
    ///
    /// index [101, 299] -> 2
    #[test]
    fn test_merge_stores() {
        let mut store1 = Store::new(300);
        let mut store2 = Store::new(200);
        for i in 500..1000 {
            store1.add(i);
            store2.add(i);
        }
        store1.merge(&store2);
        assert_eq!(store1.bins.get(0), Some(&201));
        assert_eq!(&store1.bins[1..100], vec![1u64; 99].as_slice());
        assert_eq!(store1.bins[100], 302);
        assert_eq!(&store1.bins[101..], vec![2u64; 199].as_slice());
        assert_eq!(store1.count, 1000);
    }
}
