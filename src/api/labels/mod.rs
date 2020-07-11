//! OpenTelemetry Labels
use crate::api::{KeyValue, Value};
use std::cmp::{self, Ordering};
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::sync::{Arc, Mutex};

const MAX_CONCURRENT_ENCODERS: usize = 3;
type CachedEncoders = [Option<(EncoderId, String)>; MAX_CONCURRENT_ENCODERS];

mod encoder;
pub use encoder::{default_encoder, new_encoder_id, DefaultLabelEncoder, Encoder, EncoderId};

/// Set is the representation for a distinct label set.  It manages an immutable
/// set of labels, with an internal cache for storing label encodings.
///
/// This type supports the `Equivalent` method of comparison using values of
/// type `Distinct`.
///
/// This type is used to implement:
/// 1. Metric labels
/// 2. Resource sets
/// 3. Correlation map (TODO)
#[derive(Clone, Debug, Default)]
pub struct Set {
    equivalent: Distinct,
    cached_encodings: Arc<Mutex<CachedEncoders>>,
}

impl Set {
    /// Construct a new label set form a distinct set of labels
    pub fn with_equivalent(equivalent: Distinct) -> Self {
        Set {
            equivalent,
            cached_encodings: Arc::new(Mutex::new([None, None, None])),
        }
    }

    /// The label set length.
    pub fn len(&self) -> usize {
        self.equivalent.len()
    }

    /// Returns the underlying distinct set of labels for equivalence checks..
    pub fn equivalent(&self) -> &Distinct {
        &self.equivalent
    }

    /// Check if the set of labels is empty.
    pub fn is_empty(&self) -> bool {
        self.equivalent.is_empty()
    }

    /// Iterate over the label key value pairs.
    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    /// Encode the label set with the given encoder and cache the result.
    pub fn encoded(&self, encoder: Option<&dyn Encoder>) -> String {
        if self.is_empty() || encoder.is_none() {
            return String::new();
        }
        let encoder = encoder.unwrap();

        let id = encoder.id();
        if !id.is_valid() {
            // Invalid IDs are not cached.
            return encoder.encode(&mut self.iter());
        }

        self.cached_encodings
            .lock()
            .map_or(String::new(), |mut encoders| {
                for idx in 0..MAX_CONCURRENT_ENCODERS {
                    if let Some((_, encoded)) = &encoders[idx] {
                        return encoded.clone();
                    }
                }

                let r = encoder.encode(&mut self.iter());

                for idx in 0..MAX_CONCURRENT_ENCODERS {
                    if !encoders[idx]
                        .as_ref()
                        .map_or(false, |(id, _)| id.is_valid())
                    {
                        encoders[idx] = Some((id, r.clone()));
                        return r;
                    }
                }

                // TODO: This is a performance cliff.  Find a way for this to
                // generate a warning.
                r
            })
    }
}

impl<T> From<T> for Set
where
    T: AsRef<[KeyValue]>,
{
    fn from(kvs: T) -> Self {
        let kvs = kvs.as_ref();
        if kvs.is_empty() {
            return Set::default();
        }
        let mut inner = kvs.to_vec();
        inner.sort_by(|a, b| a.key.cmp(&b.key));
        inner.dedup_by(|a, b| a.key.eq(&b.key));

        Set {
            equivalent: Distinct(inner),
            cached_encodings: Arc::new(Mutex::new([None, None, None])),
        }
    }
}

impl<'a> IntoIterator for &'a Set {
    type Item = &'a KeyValue;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.equivalent.0.iter())
    }
}
/// An iterator over the entries of a `Set`.
#[allow(missing_debug_implementations)]
pub struct Iter<'a>(std::slice::Iter<'a, KeyValue>);
impl<'a> Iterator for Iter<'a> {
    type Item = &'a KeyValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Distinct wraps a variable-size array of `kv.KeyValue`, constructed with keys
/// in sorted order. This can be used as a map key or for equality checking
/// between Sets.
#[derive(Clone, Debug, Default)]
pub struct Distinct(Vec<KeyValue>);

impl Distinct {
    /// Check if the labels are empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The length of the set of labels
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<&[KeyValue]> for Distinct {
    fn from(kvs: &[KeyValue]) -> Self {
        let mut inner = kvs.to_vec();
        inner.sort_by(|a, b| a.key.cmp(&b.key));

        Distinct(inner)
    }
}

impl Eq for Distinct {}
impl cmp::PartialEq for Distinct {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        for idx in 0..self.0.len() {
            if self.0.get(idx) != other.0.get(idx) {
                return false;
            }
        }

        true
    }
}
impl Hash for Distinct {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for kv in self.0.iter() {
            kv.key.hash(state);

            match &kv.value {
                Value::Bool(b) => b.hash(state),
                Value::I64(i) => i.hash(state),
                Value::U64(u) => u.hash(state),
                Value::F64(f) => {
                    // FIXME: f64 does not impl hash, this impl may have incorrect outcomes.
                    f.to_bits().hash(state)
                }
                Value::String(s) => s.hash(state),
                Value::Bytes(b) => state.write(b),
            }
        }
    }
}

/// Merge two iterators, yielding sorted results
pub fn merge_iters<'a, 'b, A: Iterator<Item = &'a KeyValue>, B: Iterator<Item = &'b KeyValue>>(
    a: A,
    b: B,
) -> MergeIter<'a, 'b, A, B> {
    MergeIter {
        a: a.peekable(),
        b: b.peekable(),
    }
}

/// Merge two iterators, sorting by key
#[derive(Debug)]
pub struct MergeIter<'a, 'b, A, B>
where
    A: Iterator<Item = &'a KeyValue>,
    B: Iterator<Item = &'b KeyValue>,
{
    a: Peekable<A>,
    b: Peekable<B>,
}

impl<'a, A: Iterator<Item = &'a KeyValue>, B: Iterator<Item = &'a KeyValue>> Iterator
    for MergeIter<'a, 'a, A, B>
{
    type Item = &'a KeyValue;
    fn next(&mut self) -> Option<Self::Item> {
        let which = match (self.a.peek(), self.b.peek()) {
            (Some(a), Some(b)) => Some(a.key.cmp(&b.key)),
            (Some(_), None) => Some(Ordering::Less),
            (None, Some(_)) => Some(Ordering::Greater),
            (None, None) => None,
        };

        match which {
            Some(Ordering::Less) => self.a.next(),
            Some(Ordering::Equal) => self.a.next(),
            Some(Ordering::Greater) => self.b.next(),
            None => None,
        }
    }
}
