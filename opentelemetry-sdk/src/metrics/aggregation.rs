use std::fmt;

use crate::metrics::internal::{EXPO_MAX_SCALE, EXPO_MIN_SCALE};
use opentelemetry::metrics::{MetricsError, Result};

/// The way recorded measurements are summarized.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Aggregation {
    /// An aggregation that drops all recorded data.
    Drop,

    /// An aggregation that uses the default instrument kind selection mapping to
    /// select another aggregation.
    ///
    /// A metric reader can be configured to make an aggregation selection based on
    /// instrument kind that differs from the default. This aggregation ensures the
    /// default is used.
    ///
    /// See the [DefaultAggregationSelector] for information about the default
    /// instrument kind selection mapping.
    ///
    /// [DefaultAggregationSelector]: crate::metrics::reader::DefaultAggregationSelector
    Default,

    /// An aggregation that summarizes a set of measurements as their arithmetic
    /// sum.
    Sum,

    /// An aggregation that summarizes a set of measurements as the last one made.
    LastValue,

    /// An aggregation that summarizes a set of measurements as a histogram with
    /// explicitly defined buckets.
    ExplicitBucketHistogram {
        /// The increasing bucket boundary values.
        ///
        /// Boundary values define bucket upper bounds. Buckets are exclusive of their
        /// lower boundary and inclusive of their upper bound (except at positive
        /// infinity). A measurement is defined to fall into the greatest-numbered
        /// bucket with a boundary that is greater than or equal to the measurement. As
        /// an example, boundaries defined as:
        ///
        /// vec![0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0,
        /// 1000.0, 2500.0, 5000.0, 7500.0, 10000.0];
        ///
        /// Will define these buckets:
        ///
        /// (-∞, 0], (0, 5.0], (5.0, 10.0], (10.0, 25.0], (25.0, 50.0], (50.0,
        ///  75.0], (75.0, 100.0], (100.0, 250.0], (250.0, 500.0], (500.0,
        ///  750.0], (750.0, 1000.0], (1000.0, 2500.0], (2500.0, 5000.0],
        ///  (5000.0, 7500.0], (7500.0, 10000.0], (10000.0, +∞)
        boundaries: Vec<f64>,

        /// Indicates whether to not record the min and max of the distribution.
        ///
        /// By default, these values are recorded.
        ///
        /// Recording these values for cumulative data is expected to have little
        /// value, they will represent the entire life of the instrument instead of
        /// just the current collection cycle. It is recommended to set this to
        /// `false` for that type of data to avoid computing the low-value
        /// instances.
        record_min_max: bool,
    },

    /// An aggregation that summarizes a set of measurements as a histogram with
    /// bucket widths that grow exponentially.
    Base2ExponentialHistogram {
        /// The maximum number of buckets to use for the histogram.
        max_size: u32,

        /// The maximum resolution scale to use for the histogram.
        ///
        /// The maximum value is `20`, in which case the maximum number of buckets
        /// that can fit within the range of a signed 32-bit integer index could be
        /// used.
        ///
        /// The minimum value is `-10` in which case only two buckets will be used.
        max_scale: i8,

        /// Indicates whether to not record the min and max of the distribution.
        ///
        /// By default, these values are recorded.
        ///
        /// It is generally not valuable to record min and max for cumulative data
        /// as they will represent the entire life of the instrument instead of just
        /// the current collection cycle, you can opt out by setting this value to
        /// `false`
        record_min_max: bool,
    },
}

impl fmt::Display for Aggregation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // used for stream id comparisons
        let name = match self {
            Aggregation::Drop => "Drop",
            Aggregation::Default => "Default",
            Aggregation::Sum => "Sum",
            Aggregation::LastValue => "LastValue",
            Aggregation::ExplicitBucketHistogram { .. } => "ExplicitBucketHistogram",
            Aggregation::Base2ExponentialHistogram { .. } => "Base2ExponentialHistogram",
        };

        f.write_str(name)
    }
}

impl Aggregation {
    /// Validate that this aggregation has correct configuration
    pub fn validate(&self) -> Result<()> {
        match self {
            Aggregation::Drop => Ok(()),
            Aggregation::Default => Ok(()),
            Aggregation::Sum => Ok(()),
            Aggregation::LastValue => Ok(()),
            Aggregation::ExplicitBucketHistogram { boundaries, .. } => {
                for x in boundaries.windows(2) {
                    if x[0] >= x[1] {
                        return Err(MetricsError::Config(format!(
                            "aggregation: explicit bucket histogram: non-monotonic boundaries: {:?}",
                            boundaries,
                        )));
                    }
                }

                Ok(())
            }
            Aggregation::Base2ExponentialHistogram { max_scale, .. } => {
                if *max_scale > EXPO_MAX_SCALE {
                    return Err(MetricsError::Config(format!(
                        "aggregation: exponential histogram: max scale ({}) is greater than 20",
                        max_scale,
                    )));
                }
                if *max_scale < -EXPO_MIN_SCALE {
                    return Err(MetricsError::Config(format!(
                        "aggregation: exponential histogram: max scale ({}) is less than -10",
                        max_scale,
                    )));
                }

                Ok(())
            }
        }
    }
}
