use opentelemetry::{
    global,
    metrics::{MetricsError, Result},
};
use regex::Regex;

use super::instrument::{Instrument, Stream};

fn empty_view(_inst: &Instrument) -> Option<Stream> {
    None
}

/// Used to customize the metrics that are output by the SDK.
///
/// Here are some examples when a [View] might be needed:
///
/// * Customize which Instruments are to be processed/ignored. For example, an
///   instrumented library can provide both temperature and humidity, but the
///   application developer might only want temperature.
/// * Customize the aggregation - if the default aggregation associated with the
///   [Instrument] does not meet the needs of the user. For example, an HTTP client
///   library might expose HTTP client request duration as Histogram by default,
///   but the application developer might only want the total count of outgoing
///   requests.
/// * Customize which attribute(s) are to be reported on metrics. For example,
///   an HTTP server library might expose HTTP verb (e.g. GET, POST) and HTTP
///   status code (e.g. 200, 301, 404). The application developer might only care
///   about HTTP status code (e.g. reporting the total count of HTTP requests for
///   each HTTP status code). There could also be extreme scenarios in which the
///   application developer does not need any attributes (e.g. just get the total
///   count of all incoming requests).
///
/// # Example Custom View
///
/// View is implemented for all `Fn(&Instrument) -> Option<Stream>`.
///
/// ```
/// use opentelemetry_sdk::metrics::{Instrument, MeterProvider, Stream};
///
/// // return streams for the given instrument
/// let my_view = |i: &Instrument| {
///   // return Some(Stream) or
///   None
/// };
///
/// let provider = MeterProvider::builder().with_view(my_view).build();
/// # drop(provider)
/// ```
pub trait View: Send + Sync + 'static {
    /// Defines how data should be collected for certain instruments.
    ///
    /// Return [Stream] to use for matching [Instrument]s,
    /// otherwise if there is no match, return `None`.
    fn match_inst(&self, inst: &Instrument) -> Option<Stream>;
}

impl<T> View for T
where
    T: Fn(&Instrument) -> Option<Stream> + Send + Sync + 'static,
{
    fn match_inst(&self, inst: &Instrument) -> Option<Stream> {
        self(inst)
    }
}

impl View for Box<dyn View> {
    fn match_inst(&self, inst: &Instrument) -> Option<Stream> {
        (**self).match_inst(inst)
    }
}

/// Creates a [View] that applies the [Stream] mask for all instruments that
/// match criteria.
///
/// The returned [View] will only apply the mask if all non-empty fields of
/// criteria match the corresponding [Instrument] passed to the view. If all
/// fields of the criteria are their default values, a view that matches no
/// instruments is returned. If you need to match an empty-value field, create a
/// [View] directly.
///
/// The [Instrument::name] field of criteria supports wildcard pattern matching.
/// The wildcard `*` is recognized as matching zero or more characters, and `?`
/// is recognized as matching exactly one character. For example, a pattern of
/// `*` will match all instrument names.
///
/// The [Stream] mask only applies updates for non-empty fields. By default, the
/// [Instrument] the [View] matches against will be use for the name,
/// description, and unit of the returned [Stream] and no `aggregation` or
/// `allowed_attribute_keys` are set. All non-empty fields of mask are used
/// instead of the default. If you need to set a an empty value in the returned
/// stream, create a custom [View] directly.
///
/// # Example
///
/// ```
/// use opentelemetry_sdk::metrics::{new_view, Aggregation, Instrument, Stream};
///
/// let criteria = Instrument::new().name("counter_*");
/// let mask = Stream::new().aggregation(Aggregation::Sum);
///
/// let view = new_view(criteria, mask);
/// # drop(view);
/// ```
pub fn new_view(criteria: Instrument, mask: Stream) -> Result<Box<dyn View>> {
    if criteria.is_empty() {
        global::handle_error(MetricsError::Config(format!(
            "no criteria provided, dropping view. mask: {mask:?}"
        )));
        return Ok(Box::new(empty_view));
    }
    let contains_wildcard = criteria.name.contains(|c| c == '*' || c == '?');
    let err_msg_criteria = criteria.clone();

    let match_fn: Box<dyn Fn(&Instrument) -> bool + Send + Sync> = if contains_wildcard {
        if mask.name != "" {
            global::handle_error(MetricsError::Config(format!(
				"name replacement for multiple instruments, dropping view, criteria: {criteria:?}, mask: {mask:?}"
			)));
            return Ok(Box::new(empty_view));
        }

        let pattern = criteria
            .name
            .trim_start_matches('^')
            .trim_end_matches('$')
            .replace('?', ".")
            .replace('*', ".*");
        let re =
            Regex::new(&format!("^{pattern}$")).map_err(|e| MetricsError::Config(e.to_string()))?;
        Box::new(move |i| {
            re.is_match(&i.name)
                && criteria.matches_description(i)
                && criteria.matches_kind(i)
                && criteria.matches_unit(i)
                && criteria.matches_scope(i)
        })
    } else {
        Box::new(move |i| criteria.matches(i))
    };

    let mut agg = None;
    if let Some(ma) = &mask.aggregation {
        match ma.validate() {
            Ok(_) => agg = Some(ma.clone()),
            Err(err) => {
                global::handle_error(MetricsError::Other(format!(
                    "{}, not using aggregation with view. criteria: {:?}, mask: {:?}",
                    err, err_msg_criteria, mask
                )));
            }
        }
    }

    Ok(Box::new(move |i: &Instrument| -> Option<Stream> {
        if match_fn(i) {
            Some(Stream {
                name: if !mask.name.is_empty() {
                    mask.name.clone()
                } else {
                    i.name.clone()
                },
                description: if !mask.description.is_empty() {
                    mask.description.clone()
                } else {
                    i.description.clone()
                },
                unit: if !mask.unit.as_str().is_empty() {
                    mask.unit.clone()
                } else {
                    i.unit.clone()
                },
                aggregation: agg.clone(),
                allowed_attribute_keys: mask.allowed_attribute_keys.clone(),
            })
        } else {
            None
        }
    }))
}
