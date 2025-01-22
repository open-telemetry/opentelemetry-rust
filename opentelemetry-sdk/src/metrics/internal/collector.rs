use opentelemetry::KeyValue;

use crate::metrics::{
    data::{Aggregation, AggregationDataPoints},
    Temporality,
};

use super::{
    aggregate::{AggregateTime, AttributeSetFilter},
    AggregateTimeInitiator, Aggregator, InitAggregationData, ValueMap,
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

/// Higher level abstraction (compared to [`AggregateMap`]) that also does the filtering and collection into aggregation data
pub(crate) trait AggregateCollector: Send + Sync + 'static {
    const TEMPORALITY: Temporality;
    type Aggr: Aggregator;

    fn measure(&self, value: <Self::Aggr as Aggregator>::PreComputedValue, attributes: &[KeyValue]);

    fn collect<InitAggregate, F>(
        &self,
        aggregate: &InitAggregate,
        dest: Option<&mut dyn Aggregation>,
        create_point: F,
    ) -> (usize, Option<Box<dyn Aggregation>>)
    where
        InitAggregate: InitAggregationData,
        F: FnMut(
            Vec<KeyValue>,
            &Self::Aggr,
        ) -> <InitAggregate::Aggr as AggregationDataPoints>::DataPoint;
}

pub(crate) struct Collector<AM> {
    filter: AttributeSetFilter,
    aggregate_map: AM,
    time: AggregateTimeInitiator,
}

impl<AM> Collector<AM>
where
    AM: AggregateMap,
{
    pub(crate) fn new(filter: AttributeSetFilter, aggregate_map: AM) -> Self {
        Self {
            filter,
            aggregate_map,
            time: AggregateTimeInitiator::default(),
        }
    }

    fn init_time(&self) -> AggregateTime {
        if let Temporality::Delta = AM::TEMPORALITY {
            self.time.delta()
        } else {
            self.time.cumulative()
        }
    }
}

impl<AM> AggregateCollector for Collector<AM>
where
    AM: AggregateMap,
{
    const TEMPORALITY: Temporality = AM::TEMPORALITY;

    type Aggr = AM::Aggr;

    fn measure(&self, value: <AM::Aggr as Aggregator>::PreComputedValue, attributes: &[KeyValue]) {
        self.filter.apply(attributes, |filtered_attrs| {
            self.aggregate_map.measure(value, filtered_attrs);
        });
    }

    fn collect<InitAggregate, F>(
        &self,
        aggregate: &InitAggregate,
        dest: Option<&mut dyn Aggregation>,
        create_point: F,
    ) -> (usize, Option<Box<dyn Aggregation>>)
    where
        InitAggregate: InitAggregationData,
        F: FnMut(
            Vec<KeyValue>,
            &AM::Aggr,
        ) -> <InitAggregate::Aggr as AggregationDataPoints>::DataPoint,
    {
        let time = self.init_time();
        let s_data = dest.and_then(|d| d.as_mut().downcast_mut::<InitAggregate::Aggr>());
        let mut new_agg = if s_data.is_none() {
            Some(aggregate.create_new(time))
        } else {
            None
        };
        let s_data = s_data.unwrap_or_else(|| new_agg.as_mut().expect("present if s_data is none"));
        aggregate.reset_existing(s_data, time);
        self.aggregate_map
            .collect_data_points(s_data.points(), create_point);

        (
            s_data.points().len(),
            new_agg.map(|a| Box::new(a) as Box<_>),
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
