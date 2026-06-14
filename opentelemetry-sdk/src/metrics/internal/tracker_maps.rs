use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use opentelemetry::KeyValue;

use super::{Aggregator, TrackerEntry};

type AttributeKey = Arc<[KeyValue]>;

fn attribute_key(attributes: &[KeyValue]) -> AttributeKey {
    Arc::from(attributes.to_vec().into_boxed_slice())
}

/// Maps from attribute sets to their aggregator tracker entries.
///
/// Two maps back every stream:
/// * `canonical` keys are sorted and deduplicated; they are the single source
///   of truth for stream identity, counting, and export.
/// * `lookup` keys preserve the caller-provided attribute order used when each
///   tracker is first inserted, and also contain every canonical key. This lets
///   canonical-order calls and the first observed caller order hit the first
///   lookup without checking the canonical map. Canonical keys are shared
///   between maps through `Arc<[KeyValue]>`.
pub(super) struct TrackerMaps<A: Aggregator> {
    lookup: HashMap<AttributeKey, Arc<TrackerEntry<A>>>,
    canonical: HashMap<AttributeKey, Arc<TrackerEntry<A>>>,
}

/// Lazily resolves a caller-provided attribute order to its canonical key.
///
/// The original order is always available for lookup aliases. The canonical
/// key is computed only when a canonical lookup or insertion needs it, and then
/// reused for the rest of this key's lifetime.
pub(super) struct Key<'a> {
    original: &'a [KeyValue],
    canonical: OnceCell<Vec<KeyValue>>,
}

impl<'a> Key<'a> {
    pub(super) fn new(original: &'a [KeyValue]) -> Self {
        Self {
            original,
            canonical: OnceCell::new(),
        }
    }

    fn original(&self) -> &[KeyValue] {
        self.original
    }

    fn canonical(&self) -> &[KeyValue] {
        self.canonical
            .get_or_init(|| sort_and_dedup(self.original))
            .as_slice()
    }
}

fn sort_and_dedup(attributes: &[KeyValue]) -> Vec<KeyValue> {
    // Use newly allocated vec here as incoming attributes are immutable so
    // cannot sort/de-dup in-place. TODO: This allocation can be avoided by
    // leveraging a ThreadLocal vec.
    let mut sorted = attributes.to_vec();
    sorted.sort_unstable_by(|a, b| a.key.cmp(&b.key));
    sorted.dedup_by(|a, b| a.key == b.key);
    sorted
}

impl<A: Aggregator> TrackerMaps<A> {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            lookup: HashMap::with_capacity(capacity),
            canonical: HashMap::with_capacity(capacity),
        }
    }

    /// Look up a tracker by the caller-provided order, falling back to the
    /// canonical order without caching the caller-provided order.
    pub(super) fn get(&self, key: &Key<'_>) -> Option<&Arc<TrackerEntry<A>>> {
        self.lookup
            .get(key.original())
            .or_else(|| self.lookup.get(key.canonical()))
    }

    /// Insert a new tracker, aliasing both the caller-provided order and the
    /// canonical order in the lookup map. The canonical key is shared between
    /// `lookup` and `canonical` so the alias does not clone the attributes.
    pub(super) fn insert(&mut self, key: &Key<'_>, tracker: Arc<TrackerEntry<A>>) {
        let original = key.original();
        let canonical = key.canonical();
        if original != canonical {
            self.lookup
                .insert(attribute_key(original), Arc::clone(&tracker));
        }

        let canonical_key = attribute_key(canonical);
        self.lookup
            .insert(Arc::clone(&canonical_key), Arc::clone(&tracker));
        self.canonical.insert(canonical_key, tracker);
    }

    /// Returns the tracker for `key`, inserting it if absent.
    pub(super) fn get_or_insert(
        &mut self,
        key: &Key<'_>,
        make: impl FnOnce() -> Arc<TrackerEntry<A>>,
    ) -> Arc<TrackerEntry<A>> {
        if let Some(tracker) = self.get(key) {
            return Arc::clone(tracker);
        }

        self.insert(key, make());
        Arc::clone(self.get(key).expect("tracker should exist after insertion"))
    }

    /// Iterate the canonical entries — the source of truth for export.
    pub(super) fn iter(&self) -> impl Iterator<Item = (&[KeyValue], &Arc<TrackerEntry<A>>)> {
        self.canonical
            .iter()
            .map(|(attrs, tracker)| (attrs.as_ref(), tracker))
    }

    /// Remove the given entries (matched by `Arc` identity) from both maps,
    /// keeping the alias map consistent with the canonical map. Returns the
    /// number of canonical entries removed.
    pub(super) fn evict(&mut self, stale: &[Arc<TrackerEntry<A>>]) -> usize {
        if stale.is_empty() {
            return 0;
        }
        let stale_pointers: HashSet<*const TrackerEntry<A>> =
            stale.iter().map(Arc::as_ptr).collect();
        let before = self.canonical.len();
        self.canonical
            .retain(|_, tracker| !stale_pointers.contains(&Arc::as_ptr(tracker)));
        self.lookup
            .retain(|_, tracker| !stale_pointers.contains(&Arc::as_ptr(tracker)));
        before - self.canonical.len()
    }

    /// Clear the alias map and take the canonical map for draining.
    pub(super) fn take(&mut self) -> HashMap<AttributeKey, Arc<TrackerEntry<A>>> {
        self.lookup.clear();
        std::mem::take(&mut self.canonical)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    use super::*;

    struct TestAggregator;

    impl Aggregator for TestAggregator {
        type InitConfig = ();
        type PreComputedValue = ();

        fn create(_: &Self::InitConfig) -> Self {
            Self
        }

        fn update(&self, _: Self::PreComputedValue) {}

        fn clone_and_reset(&self, _: &Self::InitConfig) -> Self {
            Self
        }
    }

    fn tracker() -> Arc<TrackerEntry<TestAggregator>> {
        Arc::new(TrackerEntry {
            aggregator: TestAggregator,
            has_been_updated: AtomicBool::new(false),
            bound_count: AtomicUsize::new(0),
        })
    }

    #[test]
    fn get_returns_tracker_for_existing_key() {
        let mut maps = TrackerMaps::with_capacity(1);
        let attributes = vec![KeyValue::new("a", "value_a")];
        let tracker = tracker();
        let key = Key::new(attributes.as_slice());
        maps.insert(&key, Arc::clone(&tracker));

        let resolved = maps.get(&key).expect("existing key should resolve");

        assert!(Arc::ptr_eq(resolved, &tracker));
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let maps = TrackerMaps::<TestAggregator>::with_capacity(1);
        let attributes = vec![KeyValue::new("a", "value_a")];
        let key = Key::new(attributes.as_slice());

        assert!(maps.get(&key).is_none());
    }

    #[test]
    fn get_returns_for_non_canonical_key() {
        let mut maps = TrackerMaps::with_capacity(1);
        let original = vec![
            KeyValue::new("b", "value_b"),
            KeyValue::new("b", "value_b"),
            KeyValue::new("a", "value_a"),
        ];
        let canonical = vec![KeyValue::new("a", "value_a"), KeyValue::new("b", "value_b")];
        let tracker = tracker();
        maps.insert(&Key::new(canonical.as_slice()), Arc::clone(&tracker));

        let original_key = Key::new(original.as_slice());
        let stored_tracker = maps
            .get(&original_key)
            .expect("canonical key should resolve");
        assert!(Arc::ptr_eq(stored_tracker, &tracker));
    }

    #[test]
    fn insert_stores_canonical_original_keys() {
        let mut maps = TrackerMaps::with_capacity(1);
        let original = vec![
            KeyValue::new("b", "value_b"),
            KeyValue::new("b", "value_b"),
            KeyValue::new("a", "value_a"),
        ];
        let canonical = vec![KeyValue::new("a", "value_a"), KeyValue::new("b", "value_b")];
        let tracker = tracker();

        let original_key = Key::new(original.as_slice());
        maps.insert(&original_key, Arc::clone(&tracker));

        let stored_original = maps
            .get(&original_key)
            .expect("existing key should resolve");
        assert!(Arc::ptr_eq(stored_original, &tracker));

        let canonical_key = Key::new(canonical.as_slice());
        let stored_canonical = maps
            .get(&canonical_key)
            .expect("existing key should resolve");
        assert!(Arc::ptr_eq(stored_canonical, &tracker));
    }

    #[test]
    fn get_or_insert_returns_tracker_for_existing_key() {
        let mut maps = TrackerMaps::with_capacity(1);
        let attributes = vec![KeyValue::new("a", "value_a")];
        let tracker = tracker();
        let key = Key::new(attributes.as_slice());
        maps.insert(&key, Arc::clone(&tracker));

        let resolved = maps.get_or_insert(&key, || panic!("existing tracker should be reused"));

        assert!(Arc::ptr_eq(&resolved, &tracker));
    }

    #[test]
    fn get_or_insert_reuses_existing_canonical_tracker() {
        let mut maps = TrackerMaps::with_capacity(1);
        let original = vec![
            KeyValue::new("b", "value_b"),
            KeyValue::new("b", "value_b"),
            KeyValue::new("a", "value_a"),
        ];
        let canonical = vec![KeyValue::new("a", "value_a"), KeyValue::new("b", "value_b")];
        let tracker = tracker();
        maps.insert(&Key::new(canonical.as_slice()), Arc::clone(&tracker));

        let original_key = Key::new(original.as_slice());
        let resolved = maps.get_or_insert(&original_key, || {
            panic!("canonical tracker should already exist")
        });

        assert!(Arc::ptr_eq(&resolved, &tracker));
        assert_eq!(maps.lookup.len(), 1);
        assert_eq!(maps.canonical.len(), 1);
    }

    #[test]
    fn get_or_insert_inserts_tracker_for_missing_key() {
        let mut maps = TrackerMaps::with_capacity(1);
        let tracker = tracker();
        let kv = vec![KeyValue::new("a", "value_a")];
        let key = Key::new(kv.as_slice());

        let resolved = maps.get_or_insert(&key, || Arc::clone(&tracker));
        assert!(Arc::ptr_eq(&resolved, &tracker));
    }

    #[test]
    fn iter_returns_only_canonical_entries() {
        let mut maps = TrackerMaps::with_capacity(1);
        let original = vec![
            KeyValue::new("b", "value_b"),
            KeyValue::new("b", "value_b"),
            KeyValue::new("a", "value_a"),
        ];
        let canonical = vec![KeyValue::new("a", "value_a"), KeyValue::new("b", "value_b")];
        let tracker = tracker();

        maps.insert(&Key::new(original.as_slice()), Arc::clone(&tracker));

        let entries: Vec<_> = maps.iter().collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, canonical.as_slice());
        assert!(Arc::ptr_eq(entries[0].1, &tracker));
    }

    #[test]
    fn evict_removes_stale_tracker_and_aliases() {
        let mut maps = TrackerMaps::with_capacity(2);
        let stale_original = vec![
            KeyValue::new("b", "value_b"),
            KeyValue::new("b", "value_b"),
            KeyValue::new("a", "value_a"),
        ];
        let stale_canonical = vec![KeyValue::new("a", "value_a"), KeyValue::new("b", "value_b")];
        let retained = vec![KeyValue::new("c", "value_c")];
        let stale_tracker = tracker();
        let retained_tracker = tracker();

        let stale_key = Key::new(stale_original.as_slice());
        let retained_key = Key::new(retained.as_slice());
        maps.insert(&stale_key, Arc::clone(&stale_tracker));
        maps.insert(&retained_key, Arc::clone(&retained_tracker));

        assert_eq!(maps.lookup.len(), 3);
        assert_eq!(maps.canonical.len(), 2);

        let removed = maps.evict(&[Arc::clone(&stale_tracker)]);

        assert_eq!(removed, 1);
        assert_eq!(maps.lookup.len(), 1);
        assert_eq!(maps.canonical.len(), 1);
        assert!(maps.get(&stale_key).is_none());
        assert!(maps.get(&Key::new(stale_canonical.as_slice())).is_none());
        assert!(Arc::ptr_eq(
            maps.get(&retained_key)
                .expect("retained tracker should remain"),
            &retained_tracker
        ));
    }

    #[test]
    fn take_returns_canonical_entries_and_clears_maps() {
        let mut maps = TrackerMaps::with_capacity(2);
        let original = vec![
            KeyValue::new("b", "value_b"),
            KeyValue::new("b", "value_b"),
            KeyValue::new("a", "value_a"),
        ];
        let canonical = vec![KeyValue::new("a", "value_a"), KeyValue::new("b", "value_b")];
        let other = vec![KeyValue::new("c", "value_c")];
        let first_tracker = tracker();
        let other_tracker = tracker();

        let original_key = Key::new(original.as_slice());
        let other_key = Key::new(other.as_slice());
        maps.insert(&original_key, Arc::clone(&first_tracker));
        maps.insert(&other_key, Arc::clone(&other_tracker));

        assert_eq!(maps.lookup.len(), 3);
        assert_eq!(maps.canonical.len(), 2);

        let taken = maps.take();

        assert!(maps.lookup.is_empty());
        assert!(maps.canonical.is_empty());
        assert_eq!(taken.len(), 2);
        assert!(Arc::ptr_eq(
            taken
                .get(canonical.as_slice())
                .expect("canonical entry should be returned"),
            &first_tracker
        ));
        assert!(Arc::ptr_eq(
            taken
                .get(other.as_slice())
                .expect("other canonical entry should be returned"),
            &other_tracker
        ));
    }
}
