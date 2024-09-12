use std::{
    collections::HashMap,
    fmt::Debug,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};

use opentelemetry::{global, metrics::MetricsError, KeyValue};

use crate::metrics::AttributeSet;

use super::{
    aggregate::is_under_cardinality_limit, Number, STREAM_OVERFLOW_ATTRIBUTES,
    STREAM_OVERFLOW_ATTRIBUTES_ERR,
};

/// Aggregator interface
pub(crate) trait Aggregator<T>: Debug + Clone
where
    T: Number,
{
    /// A static configuration that is needed by configurators.
    /// E.g. bucket_size at creation time and buckets list at aggregator update.
    type Config;

    /// Called everytime a new attribute-set is stored.
    fn create(init: &Self::Config) -> Self;

    /// Called for each measurement.
    fn update(&mut self, config: &Self::Config, measurement: T);
}

/// hashing and sorting is expensive, so we have two lists
/// sorted list is mainly needed for fast collection phase
struct WithAttribsAggregators<A> {
    // put all attribute combinations in this list
    all: HashMap<Vec<KeyValue>, Arc<Mutex<A>>>,
    sorted: HashMap<Vec<KeyValue>, Arc<Mutex<A>>>,
}

/// This class is responsible for two things:
/// * send measurement information for specific aggregator (per attribute-set)
/// * collect all attribute-sets + aggregators (either readonly OR reset)
///
/// Even though it's simple to understand it's responsibility,
/// implementation is a lot more complex to make it very performant.
pub(crate) struct AttributeSetAggregation<T, A>
where
    T: Number,
    A: Aggregator<T>,
{
    /// Aggregator for values with no attributes attached.
    no_attribs: Mutex<Option<A>>,
    list: RwLock<WithAttribsAggregators<A>>,
    /// Configuration required to create and update the [`Aggregator`]
    config: A::Config,
}

impl<T, A> AttributeSetAggregation<T, A>
where
    T: Number,
    A: Aggregator<T>,
{
    /// Initiate aggregators by specifing [`Aggregator`] configuration.
    pub(crate) fn new(init_data: A::Config) -> Self {
        Self {
            no_attribs: Mutex::new(None),
            list: RwLock::new(WithAttribsAggregators {
                all: Default::default(),
                sorted: Default::default(),
            }),
            config: init_data,
        }
    }

    /// Update specific aggregator depending on provided attributes.
    pub(crate) fn measure(&self, attrs: &[KeyValue], measurement: T) {
        if attrs.is_empty() {
            if let Ok(mut aggr) = self.no_attribs.lock() {
                aggr.get_or_insert_with(|| A::create(&self.config))
                    .update(&self.config, measurement);
            }
            return;
        }
        let Ok(list) = self.list.read() else {
            return;
        };
        if let Some(aggr) = list.all.get(attrs) {
            if let Ok(mut aggr) = aggr.lock() {
                aggr.update(&self.config, measurement);
            }
            return;
        }
        drop(list);
        let Ok(mut list) = self.list.write() else {
            return;
        };

        // Recheck again in case another thread already inserted
        if let Some(aggr) = list.all.get(attrs) {
            if let Ok(mut aggr) = aggr.lock() {
                aggr.update(&self.config, measurement);
            }
        } else if is_under_cardinality_limit(list.all.len()) {
            let mut aggr = A::create(&self.config);
            aggr.update(&self.config, measurement);
            let aggr = Arc::new(Mutex::new(aggr));
            list.all.insert(attrs.into(), aggr.clone());
            let sorted_attribs = AttributeSet::from(attrs).into_vec();
            list.sorted.insert(sorted_attribs, aggr);
        } else if let Some(aggr) = list.sorted.get(STREAM_OVERFLOW_ATTRIBUTES.deref()) {
            if let Ok(mut aggr) = aggr.lock() {
                aggr.update(&self.config, measurement);
            }
        } else {
            let mut aggr = A::create(&self.config);
            aggr.update(&self.config, measurement);
            list.sorted.insert(
                STREAM_OVERFLOW_ATTRIBUTES.clone(),
                Arc::new(Mutex::new(aggr)),
            );
            global::handle_error(MetricsError::Other(STREAM_OVERFLOW_ATTRIBUTES_ERR.into()));
        }
    }

    /// Iterate through all attribute sets and populate `DataPoints`in readonly mode.
    pub(crate) fn collect_readonly<Res, MapFn>(&self, dest: &mut Vec<Res>, mut map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, A) -> Res,
    {
        let Ok(list) = self.list.read() else {
            return;
        };
        prepare_data(dest, list.sorted.len());
        if let Ok(aggr) = self.no_attribs.lock() {
            if let Some(aggr) = aggr.deref() {
                dest.push(map_fn(Default::default(), aggr.clone()));
            }
        };
        dest.extend(
            list.sorted
                .iter()
                .filter_map(|(k, v)| v.lock().ok().map(|v| map_fn(k.clone(), v.clone()))),
        )
    }

    /// Iterate through all attribute sets and populate `DataPoints`, while also consuming (reseting) aggregators
    pub(crate) fn collect_and_reset<Res, MapFn>(&self, dest: &mut Vec<Res>, mut map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, A) -> Res,
    {
        let Ok(mut list) = self.list.write() else {
            return;
        };
        prepare_data(dest, list.sorted.len());
        if let Ok(mut aggr) = self.no_attribs.lock() {
            if let Some(aggr) = aggr.take() {
                dest.push(map_fn(Default::default(), aggr));
            }
        };
        list.all.clear();
        dest.extend(list.sorted.drain().filter_map(|(k, v)| {
            Arc::try_unwrap(v)
                .expect("this is last instance, so we cannot fail to get it")
                .into_inner()
                .ok()
                .map(|v| map_fn(k, v))
        }));
    }

    pub(crate) fn config(&self) -> &A::Config {
        &self.config
    }
}

/// Clear and allocate exactly required amount of space for all attribute-sets
fn prepare_data<T>(data: &mut Vec<T>, list_len: usize) {
    data.clear();
    let total_len = list_len + 1; // to account for no_attributes case
    if total_len > data.capacity() {
        data.reserve_exact(total_len - data.capacity());
    }
}
