//! OpenTelemetry Dynatrace Metrics
use crate::transform::common::get_time;
use opentelemetry::attributes::merge_iters;
use opentelemetry::metrics::{MetricsError, Number, NumberKind};
use opentelemetry::sdk::export::metrics::{
    Count, ExportKind, ExportKindFor, Histogram as SdkHistogram, LastValue, Max, Min, Points,
    Record, Sum as SdkSum,
};
use opentelemetry::sdk::metrics::aggregators::{
    ArrayAggregator, HistogramAggregator, LastValueAggregator, MinMaxSumCountAggregator,
    SumAggregator,
};
use opentelemetry::{Key, KeyValue, Value};
use std::borrow::Cow;
use std::cmp;
use std::collections::{btree_map, BTreeMap};
use std::fmt;
use std::fmt::Write;
use std::iter::{self, FromIterator};

/// Source of the metric data.
const METRICS_SOURCE: Key = Key::from_static_str("dt.metrics.source");

/// Dynatrace metric ingestion protocol line key.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MetricKey(Cow<'static, str>);

impl MetricKey {
    /// Create a new `MetricKey`.
    pub fn new<S: Into<Cow<'static, str>>>(value: S) -> Self {
        MetricKey(value.into())
    }

    /// Create a new const `MetricKey`.
    pub const fn from_static_str(value: &'static str) -> Self {
        MetricKey(Cow::Borrowed(value))
    }

    /// Returns a reference to the underlying key name.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<&'static str> for MetricKey {
    /// Convert a `&str` to a `MetricKey`.
    fn from(key_str: &'static str) -> Self {
        MetricKey(Cow::from(key_str))
    }
}

impl From<String> for MetricKey {
    /// Convert a `String` to a `MetricKey`.
    fn from(string: String) -> Self {
        MetricKey(Cow::from(string))
    }
}

impl From<MetricKey> for String {
    /// Converts `MetricKey` instances into `String`.
    fn from(key: MetricKey) -> Self {
        key.0.into_owned()
    }
}

impl fmt::Display for MetricKey {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prev_char_underscore = false;
        let mut escaped = self
            .0
            .as_ref()
            .chars()
            .filter_map(|c| {
                if c == '.' || c == '-' || c == '_' || c.is_numeric() || c.is_ascii_alphabetic() {
                    prev_char_underscore = false;
                    Some(c)
                } else if !prev_char_underscore {
                    prev_char_underscore = true;
                    Some('_')
                } else {
                    None
                }
            })
            .peekable();

        // The maximum metric key length is 250 characters
        if escaped
            .peek()
            .map_or(false, |c| c == &'_' || c.is_ascii_alphabetic())
        {
            fmt.write_str(&escaped.take(250).collect::<String>())?;
        } else {
            // The metric key starts with a non-ASCII alphabetic character and needs to be prefixed
            // with an underscore
            fmt.write_str(&"_".chars().chain(escaped.take(249)).collect::<String>())?;
        }

        Ok(())
    }
}

/// An immutable set of distinct metric dimensions.
#[derive(Clone, Debug, Default)]
pub struct DimensionSet {
    dimensions: BTreeMap<Key, Value>,
}

impl DimensionSet {
    /// The dimension set length.
    pub fn len(&self) -> usize {
        self.dimensions.len()
    }

    /// Check if the set of dimensions is empty.
    pub fn is_empty(&self) -> bool {
        self.dimensions.is_empty()
    }

    /// Iterate over the dimension key value pairs.
    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
    }
}

impl fmt::Display for DimensionSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let val = self
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (idx, (key, value))| {
                let offset = acc.len();
                if idx > 0 {
                    acc.push(',')
                }

                let mut prev_char_underscore = false;

                // The maximum dimension key length is 100 characters
                let key = key
                    .as_str()
                    .chars()
                    .filter_map(|c| {
                        if c == '.'
                            || c == '-'
                            || c == '_'
                            || c.is_numeric()
                            || c.is_ascii_alphabetic()
                        {
                            prev_char_underscore = false;
                            Some(c)
                        } else if !prev_char_underscore {
                            prev_char_underscore = true;
                            Some('_')
                        } else {
                            None
                        }
                    })
                    .take(100)
                    .collect::<String>()
                    .to_lowercase();

                if write!(acc, "{}", key).is_err() {
                    acc.truncate(offset);
                    return acc;
                }

                acc.push('=');

                prev_char_underscore = false;

                // The maximum dimension value length is 255 characters
                let value = value
                    .as_str()
                    .chars()
                    .filter_map(|c| {
                        if c.is_numeric() || c.is_ascii() {
                            prev_char_underscore = false;
                            Some(c)
                        } else if !prev_char_underscore {
                            prev_char_underscore = true;
                            Some('_')
                        } else {
                            None
                        }
                    })
                    .take(255)
                    .collect::<String>();

                if write!(acc, "{}", value).is_err() {
                    acc.truncate(offset);
                    return acc;
                }

                acc
            });

        fmt.write_str(&val)?;

        Ok(())
    }
}

impl PartialEq for DimensionSet {
    fn eq(&self, other: &Self) -> bool {
        self.dimensions.iter().eq(other.iter())
    }
}

impl From<Vec<KeyValue>> for DimensionSet {
    fn from(collection: Vec<KeyValue>) -> Self {
        DimensionSet {
            dimensions: collection
                .into_iter()
                .map(|kv| (kv.key, kv.value))
                .collect(),
        }
    }
}

impl From<Vec<(Key, Value)>> for DimensionSet {
    fn from(collection: Vec<(Key, Value)>) -> Self {
        let mut dimensions = BTreeMap::new();
        for (key, value) in collection.into_iter() {
            dimensions.insert(key, value);
        }
        DimensionSet { dimensions }
    }
}

impl FromIterator<KeyValue> for DimensionSet {
    fn from_iter<I: IntoIterator<Item = KeyValue>>(iter: I) -> Self {
        let mut dimensions = BTreeMap::new();
        for kv in iter {
            dimensions.insert(kv.key, kv.value);
        }
        DimensionSet { dimensions }
    }
}

impl FromIterator<(Key, Value)> for DimensionSet {
    fn from_iter<I: IntoIterator<Item = (Key, Value)>>(iter: I) -> Self {
        let mut dimensions = BTreeMap::new();
        for (key, value) in iter {
            dimensions.insert(key, value);
        }
        DimensionSet { dimensions }
    }
}

/// An iterator over the entries of a `DimensionSet`.
#[derive(Debug)]
pub struct Iter<'a>(btree_map::Iter<'a, Key, Value>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a DimensionSet {
    type Item = (&'a Key, &'a Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.dimensions.iter())
    }
}

/// [Dynatrace metrics ingestion protocol line].
///
/// [Dynatrace metrics ingestion protocol line]: https://www.dynatrace.com/support/help/how-to-use-dynatrace/metrics/metric-ingestion/metric-ingestion-protocol/
#[derive(Clone, Debug)]
pub struct MetricLine {
    kind: NumberKind,
    key: MetricKey,
    dimensions: Option<DimensionSet>,
    min: Option<Number>,
    max: Option<Number>,
    sum: Option<Number>,
    count: Option<u64>,
    delta: Option<Number>,
    gauge: Option<Number>,
    timestamp: Option<u64>,
}

impl MetricLine {
    /// Create a new `MetricLine`.
    pub fn new(key: MetricKey, kind: NumberKind) -> Self {
        MetricLine {
            key,
            kind,
            dimensions: None,
            min: None,
            max: None,
            sum: None,
            count: None,
            delta: None,
            gauge: None,
            timestamp: None,
        }
    }

    /// Common attributes that apply to this metric line.
    pub fn dimensions(mut self, dimensions: Option<DimensionSet>) -> Self {
        self.dimensions = dimensions;
        self
    }

    /// The min value.
    pub fn min(mut self, min: Option<Number>) -> Self {
        self.min = min;
        self
    }

    /// The max value.
    pub fn max(mut self, max: Option<Number>) -> Self {
        self.max = max;
        self
    }

    /// The sum value.
    pub fn sum(mut self, sum: Option<Number>) -> Self {
        self.sum = sum;
        self
    }

    /// The count value.
    pub fn count(mut self, count: Option<u64>) -> Self {
        self.count = count;
        self
    }

    /// The delta value.
    pub fn delta(mut self, delta: Option<Number>) -> Self {
        self.delta = delta;
        self
    }

    /// The gauge value.
    pub fn gauge(mut self, gauge: Option<Number>) -> Self {
        self.gauge = gauge;
        self
    }

    /// The timestamp in UTC milliseconds.
    /// Allowed range is between 1 hour into the past and 10 minutes into the future from now.
    /// If no timestamp is provided, the ingestion time of the Dynatrace server will be used automatically.
    pub fn timestamp(mut self, timestamp: Option<u64>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

impl fmt::Display for MetricLine {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&format!("{}", &self.key))?;
        fmt.write_char(',')?;

        if let Some(dimensions) = self.dimensions.to_owned() {
            fmt.write_str(&format!("{}", dimensions))?;
        }

        if self.min.is_some() && self.max.is_some() && self.sum.is_some() && self.count.is_some() {
            let min = self.min.to_owned().unwrap();
            let max = self.max.to_owned().unwrap();
            let sum = self.sum.to_owned().unwrap();

            fmt.write_char(' ')?;
            if min.partial_cmp(&self.kind, &sum) == Some(cmp::Ordering::Equal)
                && max.partial_cmp(&self.kind, &sum) == Some(cmp::Ordering::Equal)
                && self.count == Some(1)
            {
                fmt.write_fmt(format_args!("gauge,{}", convert(&self.kind, sum)))?;
            } else {
                fmt.write_fmt(format_args!(
                    "gauge,min={},max={},sum={},count={}",
                    convert(&self.kind, min),
                    convert(&self.kind, max),
                    sum.to_f64(&self.kind),
                    self.count.to_owned().unwrap(),
                ))?;
            }
        } else if let Some(delta) = self.delta.to_owned() {
            fmt.write_char(' ')?;
            fmt.write_fmt(format_args!("count,delta={}", convert(&self.kind, delta)))?;
        } else if let Some(gauge) = self.gauge.to_owned() {
            fmt.write_char(' ')?;
            fmt.write_fmt(format_args!("gauge,{}", convert(&self.kind, gauge)))?;
        }

        if let Some(timestamp) = self.timestamp.to_owned() {
            fmt.write_char(' ')?;
            fmt.write_str(&timestamp.to_string())?;
        }

        Ok(())
    }
}

impl PartialEq for MetricLine {
    fn eq(&self, other: &Self) -> bool {
        self.kind.eq(&other.kind)
            && self.key.eq(&other.key)
            && match (self.dimensions.clone(), other.dimensions.clone()) {
                (Some(a), Some(b)) => a.eq(&b),
                (None, None) => true,
                _ => false,
            }
            && match (self.min.clone(), other.min.clone()) {
                (Some(a), Some(b)) => a.partial_cmp(&self.kind, &b) == Some(cmp::Ordering::Equal),
                (None, None) => true,
                _ => false,
            }
            && match (self.max.clone(), other.max.clone()) {
                (Some(a), Some(b)) => a.partial_cmp(&self.kind, &b) == Some(cmp::Ordering::Equal),
                (None, None) => true,
                _ => false,
            }
            && match (self.sum.clone(), other.sum.clone()) {
                (Some(a), Some(b)) => a.partial_cmp(&self.kind, &b) == Some(cmp::Ordering::Equal),
                (None, None) => true,
                _ => false,
            }
            && self.count.eq(&other.count)
            && match (self.delta.clone(), other.delta.clone()) {
                (Some(a), Some(b)) => a.partial_cmp(&self.kind, &b) == Some(cmp::Ordering::Equal),
                (None, None) => true,
                _ => false,
            }
            && match (self.gauge.clone(), other.gauge.clone()) {
                (Some(a), Some(b)) => a.partial_cmp(&self.kind, &b) == Some(cmp::Ordering::Equal),
                (None, None) => true,
                _ => false,
            }
            && self.timestamp.eq(&other.timestamp)
    }
}

/// Transform a record to a Dynatrace metrics ingestion protocol metric line.
pub(crate) fn record_to_metric_line(
    record: &Record,
    export_selector: &dyn ExportKindFor,
    prefix: Option<String>,
    default_dimensions: Option<DimensionSet>,
    timestamp: bool,
) -> Result<Vec<MetricLine>, MetricsError> {
    let aggregator = record.aggregator().ok_or(MetricsError::NoDataCollected)?;
    let descriptor = record.descriptor();

    let kind = descriptor.number_kind();

    let key = if prefix.is_some() {
        MetricKey::new(format!("{}.{}", prefix.unwrap(), descriptor.name()))
    } else {
        MetricKey::new(descriptor.name().to_string())
    };

    let source_key = METRICS_SOURCE;
    let source_value = Value::String("opentelemetry".into());

    let iter = record
        .attributes()
        .iter()
        .chain(iter::once((&source_key, &source_value)));
    let dimensions = if let Some(default_dimensions) = default_dimensions {
        DimensionSet::from_iter(
            merge_iters(default_dimensions.iter(), iter).map(|(k, v)| (k.to_owned(), v.to_owned())),
        )
    } else {
        DimensionSet::from_iter(iter.map(|(k, v)| (k.to_owned(), v.to_owned())))
    };

    let temporality = export_selector.export_kind_for(descriptor);

    let mut metric_line_data: Vec<MetricLine> = Vec::with_capacity(1);

    if let Some(array) = aggregator.as_any().downcast_ref::<ArrayAggregator>() {
        if let Ok(points) = array.points() {
            let timestamp = if timestamp {
                Some(get_time(record.end_time().to_owned()))
            } else {
                None
            };

            metric_line_data.reserve(points.len());

            points.iter().for_each(|val| {
                metric_line_data.push(MetricLine {
                    kind: kind.clone(),
                    key: key.clone(),
                    dimensions: Some(dimensions.clone()),
                    min: None,
                    max: None,
                    sum: None,
                    count: None,
                    delta: None,
                    gauge: Some(val.to_owned()),
                    timestamp,
                })
            })
        }
    } else if let Some(last_value) = aggregator.as_any().downcast_ref::<LastValueAggregator>() {
        let (val, sample_time) = last_value.last_value()?;
        let timestamp = if timestamp {
            Some(get_time(sample_time))
        } else {
            None
        };

        metric_line_data.push(MetricLine {
            kind: kind.to_owned(),
            key,
            dimensions: Some(dimensions),
            min: None,
            max: None,
            sum: None,
            count: None,
            delta: None,
            gauge: Some(val),
            timestamp,
        });
    } else if let Some(sum) = aggregator.as_any().downcast_ref::<SumAggregator>() {
        let val = sum.sum()?;
        let timestamp = if timestamp {
            Some(get_time(record.end_time().to_owned()))
        } else {
            None
        };

        let mut metric_line = MetricLine {
            kind: kind.to_owned(),
            key,
            dimensions: Some(dimensions),
            min: None,
            max: None,
            sum: None,
            count: None,
            delta: None,
            gauge: None,
            timestamp,
        };

        match temporality {
            ExportKind::Cumulative => metric_line.gauge = Some(val),
            ExportKind::Delta => metric_line.delta = Some(val),
        };

        metric_line_data.push(metric_line);
    } else if let Some(min_max_sum_count) = aggregator
        .as_any()
        .downcast_ref::<MinMaxSumCountAggregator>()
    {
        let (min, max, sum, count) = (
            min_max_sum_count.min()?,
            min_max_sum_count.max()?,
            min_max_sum_count.sum()?,
            min_max_sum_count.count()?,
        );
        let timestamp = if timestamp {
            Some(get_time(record.end_time().to_owned()))
        } else {
            None
        };

        metric_line_data.push(MetricLine {
            kind: kind.to_owned(),
            key,
            dimensions: Some(dimensions),
            min: Some(min),
            max: Some(max),
            sum: Some(sum),
            count: Some(count),
            delta: None,
            gauge: None,
            timestamp,
        });
    } else if let Some(histogram) = aggregator.as_any().downcast_ref::<HistogramAggregator>() {
        let (sum, count, buckets) = (histogram.sum()?, histogram.count()?, histogram.histogram()?);
        let (counts, boundaries) = (buckets.counts(), buckets.boundaries());

        let mut min_idx: i32 = -1;
        let mut max_idx: i32 = -1;

        for (i, val) in counts.iter().enumerate() {
            if val > &0.0 {
                if min_idx == -1 {
                    min_idx = i as i32;
                }
                max_idx = i as i32;
            }
        }

        let min: f64 = if min_idx == -1 {
            0.0
        } else if min_idx == 0 {
            boundaries[0]
        } else {
            boundaries[min_idx as usize - 1]
        };

        let max: f64 = if max_idx as usize == counts.len() - 1 {
            boundaries[max_idx as usize - 1]
        } else {
            boundaries[max_idx as usize]
        };

        let timestamp = if timestamp {
            Some(get_time(record.end_time().to_owned()))
        } else {
            None
        };

        metric_line_data.push(MetricLine {
            kind: NumberKind::F64,
            key,
            dimensions: Some(dimensions),
            min: Some(Number::from(min)),
            max: Some(Number::from(max)),
            sum: Some(Number::from(sum.to_f64(&NumberKind::I64))),
            count: Some(count),
            delta: None,
            gauge: None,
            timestamp,
        });
    }

    Ok(metric_line_data)
}

/// Converts the number to a string.
#[inline]
fn convert(kind: &NumberKind, number: Number) -> String {
    match &kind {
        NumberKind::U64 => number.to_u64(kind).to_string(),
        NumberKind::I64 => number.to_i64(kind).to_string(),
        NumberKind::F64 => number.to_f64(kind).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::common::get_time;
    use crate::transform::metrics::MetricLine;
    use crate::transform::record_to_metric_line;
    use opentelemetry::attributes::AttributeSet;
    use opentelemetry::metrics::{Descriptor, InstrumentKind, MetricsError, Number, NumberKind};
    use opentelemetry::sdk::export::metrics::{record, Aggregator, ExportKindSelector};
    use opentelemetry::sdk::metrics::aggregators::{
        histogram, last_value, min_max_sum_count, SumAggregator,
    };
    use opentelemetry::sdk::Resource;
    use opentelemetry::KeyValue;
    use std::borrow::Cow;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_key() {
        fn key_data() -> Vec<(&'static str, Cow<'static, str>, Cow<'static, str>)> {
            vec![
                (
                    "keep if containing _-.",
                    "value.123_foo-bar23_foo-bar".into(),
                    "value.123_foo-bar23_foo-bar".into(),
                ),
                (
                    "keep if starting with an underscore",
                    "_test".into(),
                    "_test".into(),
                ),
                (
                    "replace with an underscore if starting with a digit",
                    "0123456789".into(),
                    "_0123456789".into(),
                ),
                (
                    "add an underscore prefix if starting with /",
                    "/0123456789".into(),
                    "_0123456789".into(),
                ),
                (
                    "add an underscore prefix if starting with :",
                    ":0123456789".into(),
                    "_0123456789".into(),
                ),
                (
                    "add an underscore prefix if starting with ;",
                    ";0123456789".into(),
                    "_0123456789".into(),
                ),
                (
                    "prefix with an underscore if starting with a dot",
                    ".test".into(),
                    "_.test".into(),
                ),
                (
                    "replace with an underscore if starting with lowercase non-alphabetic character",
                    "ätest".into(),
                    "_test".into(),
                ),
                (
                    "replace with an underscore if starting with uppercase non-alphabetic character",
                    "Ätest".into(),
                    "_test".into(),
                ),
                (
                    "replace invalid characters",
                    "test/abc-123".into(),
                    "test_abc-123".into(),
                ),
                (
                    "skip consecutively following underscores",
                    "test.äöüß_123".into(),
                    "test.__123".into(),
                ),
                (
                    "skip replacing invalid characters with consecutively following underscores",
                    "test.äbc_123".into(),
                    "test._bc_123".into(),
                ),
                (
                    "limit to 250 characters",
                    "a".repeat(251).into(),
                    "a".repeat(250).into(),
                ),
                (
                    "limit to 250 characters with invalid first character",
                    format!("ä{}", "a".repeat(250)).into(),
                    format!("_{}", "a".repeat(249)).into(),
                ),
                (
                    "valid input",
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789".into(),
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789".into(),
                ),
                (
                    "valid input starting with an underscore",
                    "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789".into(),
                    "_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789".into(),
                ),
            ]
        }

        for (name, data, sanitized) in key_data() {
            assert_eq!(
                sanitized,
                format!("{}", MetricKey::new(data)),
                "{} doesn't match",
                name
            )
        }
    }

    #[test]
    fn test_record_to_metric_line() -> Result<(), MetricsError> {
        let attributes = vec![("KEY", "VALUE"), ("test.abc_123-", "value.123_foo-bar")];
        let attribute_set = AttributeSet::from_attributes(
            attributes
                .iter()
                .cloned()
                .map(|(k, v)| opentelemetry::KeyValue::new(k, v)),
        );
        let resource = Resource::new(vec![
            opentelemetry::KeyValue::new("process", "rust"),
            opentelemetry::KeyValue::new("runtime", "sync"),
        ]);
        let start_time = SystemTime::now();
        let end_time = SystemTime::now().checked_add(Duration::new(30, 0)).unwrap();

        // Array
        {
            let descriptor = Descriptor::new(
                "test_array".to_string(),
                "test",
                None,
                None,
                InstrumentKind::Counter,
                NumberKind::I64,
            );
            let aggregator = ArrayAggregator::default();
            let val = Number::from(12_i64);
            aggregator.update(&val, &descriptor)?;
            let val = Number::from(24_i64);
            aggregator.update(&val, &descriptor)?;
            let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
            let record = record(
                &descriptor,
                &attribute_set,
                &resource,
                Some(&wrapped_aggregator),
                start_time,
                end_time,
            );

            let metric_line_data =
                record_to_metric_line(&record, &ExportKindSelector::Cumulative, None, None, true)?;

            let dimensions = DimensionSet::from(vec![
                KeyValue::new("KEY", "VALUE"),
                KeyValue::new("test.abc_123-", "value.123_foo-bar"),
                KeyValue::new(METRICS_SOURCE, "opentelemetry"),
            ]);

            let expect = vec![
                MetricLine {
                    key: MetricKey::new("test_array"),
                    kind: NumberKind::I64,
                    dimensions: Some(dimensions.clone()),
                    min: None,
                    max: None,
                    sum: None,
                    count: None,
                    delta: None,
                    gauge: Some(Number::from(12_i64)),
                    timestamp: Some(get_time(end_time)),
                },
                MetricLine {
                    key: MetricKey::new("test_array"),
                    kind: NumberKind::I64,
                    dimensions: Some(dimensions),
                    min: None,
                    max: None,
                    sum: None,
                    count: None,
                    delta: None,
                    gauge: Some(Number::from(24_i64)),
                    timestamp: Some(get_time(end_time)),
                },
            ];

            assert_eq!(expect, metric_line_data);

            let mut metric_lines: Vec<String> = metric_line_data
                .iter()
                .map(|export_line| format!("{}", export_line))
                .collect();
            metric_lines.sort_unstable();

            let mut iter = metric_lines.iter();

            assert_eq!(
                Some(&format!(
                    "test_array,key=VALUE,{}={},test.abc_123-=value.123_foo-bar gauge,12 {}",
                    METRICS_SOURCE,
                    "opentelemetry",
                    get_time(end_time),
                )),
                iter.next()
            );
            assert_eq!(
                Some(&format!(
                    "test_array,key=VALUE,{}={},test.abc_123-=value.123_foo-bar gauge,24 {}",
                    METRICS_SOURCE,
                    "opentelemetry",
                    get_time(end_time),
                )),
                iter.next()
            );
            assert_eq!(None, iter.next());
        }

        // Sum
        {
            let descriptor = Descriptor::new(
                "test_sum".to_string(),
                "test",
                None,
                None,
                InstrumentKind::Counter,
                NumberKind::I64,
            );
            let aggregator = SumAggregator::default();
            let val = Number::from(12_i64);
            aggregator.update(&val, &descriptor)?;
            let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
            let record = record(
                &descriptor,
                &attribute_set,
                &resource,
                Some(&wrapped_aggregator),
                start_time,
                end_time,
            );

            // ExportKindSelector::Cumulative
            let metric_line_data =
                record_to_metric_line(&record, &ExportKindSelector::Cumulative, None, None, true)?;

            let dimensions = DimensionSet::from(vec![
                KeyValue::new("KEY", "VALUE"),
                KeyValue::new("test.abc_123-", "value.123_foo-bar"),
                KeyValue::new(METRICS_SOURCE, "opentelemetry"),
            ]);

            let expect = vec![MetricLine {
                key: MetricKey::new("test_sum"),
                kind: NumberKind::I64,
                dimensions: Some(dimensions),
                min: None,
                max: None,
                sum: None,
                count: None,
                delta: None,
                gauge: Some(Number::from(12_i64)),
                timestamp: Some(get_time(end_time)),
            }];

            assert_eq!(expect, metric_line_data);

            let mut metric_lines: Vec<String> = metric_line_data
                .iter()
                .map(|export_line| format!("{}", export_line))
                .collect();
            metric_lines.sort_unstable();

            let mut iter = metric_lines.iter();

            assert_eq!(
                Some(&format!(
                    "test_sum,key=VALUE,{}={},test.abc_123-=value.123_foo-bar gauge,12 {}",
                    METRICS_SOURCE,
                    "opentelemetry",
                    get_time(end_time),
                )),
                iter.next()
            );
            assert_eq!(None, iter.next());

            // ExportKindSelector::Delta
            let metric_line_data =
                record_to_metric_line(&record, &ExportKindSelector::Delta, None, None, true)?;

            let dimensions = DimensionSet::from(vec![
                KeyValue::new("KEY", "VALUE"),
                KeyValue::new("test.abc_123-", "value.123_foo-bar"),
                KeyValue::new(METRICS_SOURCE, "opentelemetry"),
            ]);

            let expect = vec![MetricLine {
                key: MetricKey::new("test_sum"),
                kind: NumberKind::I64,
                dimensions: Some(dimensions),
                min: None,
                max: None,
                sum: None,
                count: None,
                delta: Some(Number::from(12_i64)),
                gauge: None,
                timestamp: Some(get_time(end_time)),
            }];

            assert_eq!(expect, metric_line_data);

            let mut metric_lines: Vec<String> = metric_line_data
                .iter()
                .map(|export_line| format!("{}", export_line))
                .collect();
            metric_lines.sort_unstable();

            let mut iter = metric_lines.iter();

            assert_eq!(
                Some(&format!(
                    "test_sum,key=VALUE,{}={},test.abc_123-=value.123_foo-bar count,delta=12 {}",
                    METRICS_SOURCE,
                    "opentelemetry",
                    get_time(end_time),
                )),
                iter.next()
            );
            assert_eq!(None, iter.next());
        }

        // Last Value
        {
            let descriptor = Descriptor::new(
                "test_last_value".to_string(),
                "test",
                None,
                None,
                InstrumentKind::ValueObserver,
                NumberKind::I64,
            );
            let aggregator = last_value();
            let val1 = Number::from(12_i64);
            let val2 = Number::from(14_i64);
            aggregator.update(&val1, &descriptor)?;
            aggregator.update(&val2, &descriptor)?;
            let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
            let record = record(
                &descriptor,
                &attribute_set,
                &resource,
                Some(&wrapped_aggregator),
                start_time,
                end_time,
            );

            let metric_line_data =
                record_to_metric_line(&record, &ExportKindSelector::Cumulative, None, None, false)?;

            let dimensions = DimensionSet::from(vec![
                KeyValue::new("KEY", "VALUE"),
                KeyValue::new("test.abc_123-", "value.123_foo-bar"),
                KeyValue::new(METRICS_SOURCE, "opentelemetry"),
            ]);

            let expect = vec![MetricLine {
                key: MetricKey::new("test_last_value"),
                kind: NumberKind::I64,
                dimensions: Some(dimensions),
                min: None,
                max: None,
                sum: None,
                count: None,
                delta: None,
                gauge: Some(Number::from(14_i64)),
                timestamp: None,
            }];

            assert_eq!(expect, metric_line_data);

            let mut metric_lines: Vec<String> = metric_line_data
                .iter()
                .map(|export_line| format!("{}", export_line))
                .collect();
            metric_lines.sort_unstable();

            let mut iter = metric_lines.iter();

            assert_eq!(
                Some(&format!(
                    "test_last_value,key=VALUE,{}={},test.abc_123-=value.123_foo-bar gauge,14",
                    METRICS_SOURCE, "opentelemetry",
                )),
                iter.next()
            );
            assert_eq!(None, iter.next());
        }

        // MinMaxSumCount
        {
            let descriptor = Descriptor::new(
                "test_min_max_sum_count".to_string(),
                "test",
                None,
                None,
                InstrumentKind::UpDownSumObserver,
                NumberKind::I64,
            );
            let aggregator = min_max_sum_count(&descriptor);
            let vals = vec![1i64.into(), 2i64.into(), 3i64.into()];
            for val in vals.iter() {
                aggregator.update(val, &descriptor)?;
            }
            let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
            let record = record(
                &descriptor,
                &attribute_set,
                &resource,
                Some(&wrapped_aggregator),
                start_time,
                end_time,
            );

            let metric_line_data =
                record_to_metric_line(&record, &ExportKindSelector::Cumulative, None, None, true)?;

            let dimensions = DimensionSet::from(vec![
                KeyValue::new("KEY", "VALUE"),
                KeyValue::new("test.abc_123-", "value.123_foo-bar"),
                KeyValue::new(METRICS_SOURCE, "opentelemetry"),
            ]);

            let expect = vec![MetricLine {
                key: MetricKey::new("test_min_max_sum_count"),
                kind: NumberKind::I64,
                dimensions: Some(dimensions),
                min: Some(Number::from(1_i64)),
                max: Some(Number::from(3_i64)),
                sum: Some(Number::from(6_i64)),
                count: Some(3),
                delta: None,
                gauge: None,
                timestamp: Some(get_time(end_time)),
            }];

            assert_eq!(expect, metric_line_data);

            let mut metric_lines: Vec<String> = metric_line_data
                .iter()
                .map(|export_line| format!("{}", export_line))
                .collect();
            metric_lines.sort_unstable();

            let mut iter = metric_lines.iter();

            assert_eq!(
                Some(&format!(
                    "test_min_max_sum_count,key=VALUE,{}={},test.abc_123-=value.123_foo-bar gauge,min=1,max=3,sum=6,count=3 {}",
                    METRICS_SOURCE,
                    "opentelemetry",
                    get_time(end_time),
                )),
                iter.next()
            );
            assert_eq!(None, iter.next());
        }

        // Histogram
        {
            let descriptor = Descriptor::new(
                "test_histogram".to_string(),
                "test",
                None,
                None,
                InstrumentKind::Histogram,
                NumberKind::I64,
            );
            let bound = [0.1, 0.2, 0.3];
            let aggregator = histogram(&descriptor, &bound);
            let vals = vec![1i64.into(), 2i64.into(), 3i64.into()];
            for val in vals.iter() {
                aggregator.update(val, &descriptor)?;
            }
            let wrapped_aggregator: Arc<dyn Aggregator + Send + Sync> = Arc::new(aggregator);
            let record = record(
                &descriptor,
                &attribute_set,
                &resource,
                Some(&wrapped_aggregator),
                start_time,
                end_time,
            );

            let metric_line_data =
                record_to_metric_line(&record, &ExportKindSelector::Cumulative, None, None, true)?;

            let dimensions = DimensionSet::from(vec![
                KeyValue::new("KEY", "VALUE"),
                KeyValue::new("test.abc_123-", "value.123_foo-bar"),
                KeyValue::new(METRICS_SOURCE, "opentelemetry"),
            ]);

            let expect = vec![MetricLine {
                key: MetricKey::new("test_histogram"),
                kind: NumberKind::F64,
                dimensions: Some(dimensions),
                min: Some(Number::from(0.3_f64)),
                max: Some(Number::from(0.3_f64)),
                sum: Some(Number::from(6_f64)),
                count: Some(3),
                delta: None,
                gauge: None,
                timestamp: Some(get_time(end_time)),
            }];

            assert_eq!(expect, metric_line_data);

            let mut metric_lines: Vec<String> = metric_line_data
                .iter()
                .map(|export_line| format!("{}", export_line))
                .collect();
            metric_lines.sort_unstable();

            let mut iter = metric_lines.iter();

            assert_eq!(
                Some(&format!(
                    "test_histogram,key=VALUE,{}={},test.abc_123-=value.123_foo-bar gauge,min=0.3,max=0.3,sum=6,count=3 {}",
                    METRICS_SOURCE,
                    "opentelemetry",
                    get_time(end_time),
                )),
                iter.next()
            );
            assert_eq!(None, iter.next());
        }

        Ok(())
    }
}
