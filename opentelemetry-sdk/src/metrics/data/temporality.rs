/// Defines the window that an aggregation was calculated over.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Temporality {
    /// A measurement interval that continues to expand forward in time from a
    /// starting point.
    ///
    /// New measurements are added to all previous measurements since a start time.
    #[default]
    Cumulative,

    /// A measurement interval that resets each cycle.
    ///
    /// Measurements from one cycle are recorded independently, measurements from
    /// other cycles do not affect them.
    Delta,

    /// Configures Synchronous Counter and Histogram instruments to use
    /// Delta aggregation temporality, which allows them to shed memory
    /// following a cardinality explosion, thus use less memory.
    LowMemory,
}
