/// Defines the window that an aggregation was calculated over.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Temporality {
    /// A measurement interval that continues to expand forward in time from a
    /// starting point.
    ///
    /// New measurements are added to all previous measurements since a start time.
    Cumulative,

    /// A measurement interval that resets each cycle.
    ///
    /// Measurements from one cycle are recorded independently, measurements from
    /// other cycles do not affect them.
    Delta,
}
