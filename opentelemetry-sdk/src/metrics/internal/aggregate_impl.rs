use std::{marker::PhantomData, sync::Arc};

use opentelemetry::KeyValue;

use crate::metrics::{
    data::{Aggregation, AggregationDataPoints},
    Temporality,
};

use super::{
    aggregate::{AggregateTime, AttributeSetFilter},
    AggregateFns, AggregateTimeInitiator, Aggregator, ComputeAggregation, Measure, Number,
    ValueMap,
};

/// Aggregate measurements for attribute sets and collect these aggregates into data points for specific temporality
pub(crate) trait AggregateMap: Send + Sync + 'static {
    const TEMPORALITY: Temporality;
    type Aggr: Aggregator;

    fn measure(&self, value: <Self::Aggr as Aggregator>::PreComputedValue, attributes: &[KeyValue]);

    fn collect_data_points<DP, MapFn>(&self, dest: &mut Vec<DP>, map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, &Self::Aggr) -> DP;
}

/// This trait provides aggregation specific functionality
pub(crate) trait AggregationImpl<T>: Send + Sync + 'static {
    // an implementation that knows how to aggregate a measurement
    type Aggr: Aggregator;
    // an implementation that stores collected aggregation data
    type AggrData: Aggregation + AggregationDataPoints;

    fn precompute(&self, value: T) -> <Self::Aggr as Aggregator>::PreComputedValue;
    fn new_aggregation_data(&self, temporality: Temporality, time: AggregateTime)
        -> Self::AggrData;
    fn reset_aggregation_data(
        &self,
        existing: &mut Self::AggrData,
        temporality: Temporality,
        time: AggregateTime,
    );
    fn build_create_points_fn(
        &self,
    ) -> impl FnMut(Vec<KeyValue>, &Self::Aggr) -> <Self::AggrData as AggregationDataPoints>::DataPoint;
}

pub(crate) fn create_aggregation<A, AM, T>(
    aggregation: A,
    aggregate_map: AM,
    filter: AttributeSetFilter,
) -> AggregateFns<T>
where
    AM: AggregateMap,
    A: AggregationImpl<T, Aggr = AM::Aggr>,
    T: Number,
{
    let fns = Arc::new(AggregionFnsImpl {
        filter,
        aggregation,
        aggregate_map,
        time: AggregateTimeInitiator::default(),
        _marker: Default::default(),
    });
    AggregateFns {
        collect: fns.clone(),
        measure: fns,
    }
}

struct AggregionFnsImpl<A, AM, T> {
    filter: AttributeSetFilter,
    aggregation: A,
    aggregate_map: AM,
    time: AggregateTimeInitiator,
    _marker: PhantomData<T>,
}

impl<A, AM, T> Measure<T> for AggregionFnsImpl<A, AM, T>
where
    A: AggregationImpl<T>,
    AM: AggregateMap<Aggr = A::Aggr>,
    T: Number,
{
    fn call(&self, measurement: T, attrs: &[KeyValue]) {
        self.filter.apply(attrs, |filtered_attrs| {
            let precomputed = self.aggregation.precompute(measurement);
            self.aggregate_map.measure(precomputed, filtered_attrs);
        });
    }
}

impl<A, AM, T> ComputeAggregation for AggregionFnsImpl<A, AM, T>
where
    A: AggregationImpl<T>,
    AM: AggregateMap<Aggr = A::Aggr>,
    T: Number,
{
    fn call(&self, dest: Option<&mut dyn Aggregation>) -> (usize, Option<Box<dyn Aggregation>>) {
        let time = if let Temporality::Delta = AM::TEMPORALITY {
            self.time.delta()
        } else {
            self.time.cumulative()
        };
        let mut s_data = dest.and_then(|d| d.as_mut().downcast_mut::<A::AggrData>());
        let mut new_agg = match s_data.as_mut() {
            Some(existing) => {
                self.aggregation
                    .reset_aggregation_data(existing, AM::TEMPORALITY, time);
                None
            }
            None => Some(self.aggregation.new_aggregation_data(AM::TEMPORALITY, time)),
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));

        let create_points_fn = self.aggregation.build_create_points_fn();
        self.aggregate_map
            .collect_data_points(s_data.points(), create_points_fn);

        (
            s_data.points().len(),
            new_agg.map(|a| Box::new(a) as Box<dyn Aggregation>),
        )
    }
}

/// At the moment use [`ValueMap`] under the hood (which support both Delta and Cumulative), to implement `AggregateMap` for Delta temporality
/// Later this could be improved to support only Delta temporality
pub(crate) struct DeltaValueMap<A>(ValueMap<A>)
where
    A: Aggregator;

impl<A> DeltaValueMap<A>
where
    A: Aggregator,
{
    pub(crate) fn new(config: A::InitConfig) -> Self {
        Self(ValueMap::new(config))
    }
}

impl<A> AggregateMap for DeltaValueMap<A>
where
    A: Aggregator,
    <A as Aggregator>::InitConfig: Send + Sync,
{
    const TEMPORALITY: Temporality = Temporality::Delta;

    type Aggr = A;

    fn measure(
        &self,
        value: <Self::Aggr as Aggregator>::PreComputedValue,
        attributes: &[KeyValue],
    ) {
        self.0.measure(value, attributes);
    }

    fn collect_data_points<DP, MapFn>(&self, dest: &mut Vec<DP>, mut map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, &Self::Aggr) -> DP,
    {
        self.0
            .collect_and_reset(dest, |attributes, aggr| map_fn(attributes, &aggr));
    }
}

/// At the moment use [`ValueMap`] under the hood (which support both Delta and Cumulative), to implement `AggregateMap` for Cumulative temporality
/// Later this could be improved to support only Cumulative temporality
pub(crate) struct CumulativeValueMap<A>(ValueMap<A>)
where
    A: Aggregator;

impl<A> CumulativeValueMap<A>
where
    A: Aggregator,
{
    pub(crate) fn new(config: A::InitConfig) -> Self {
        Self(ValueMap::new(config))
    }
}

impl<A> AggregateMap for CumulativeValueMap<A>
where
    A: Aggregator,
    <A as Aggregator>::InitConfig: Send + Sync,
{
    const TEMPORALITY: Temporality = Temporality::Cumulative;

    type Aggr = A;

    fn measure(
        &self,
        value: <Self::Aggr as Aggregator>::PreComputedValue,
        attributes: &[KeyValue],
    ) {
        self.0.measure(value, attributes);
    }

    fn collect_data_points<DP, MapFn>(&self, dest: &mut Vec<DP>, map_fn: MapFn)
    where
        MapFn: FnMut(Vec<KeyValue>, &Self::Aggr) -> DP,
    {
        self.0.collect_readonly(dest, map_fn);
    }
}
