use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
use std::collections::{HashMap, HashSet};
use std::fs::File;

// Given two ResourceSpans, assert that they are equal except for the timestamps
pub struct TraceAsserter {
    results: ResourceSpans,
    expected: ResourceSpans,
}

impl TraceAsserter {
    // Create a new TraceAsserter
    pub fn new(results: ResourceSpans, expected: ResourceSpans) -> Self {
        TraceAsserter { results, expected }
    }

    pub fn assert(self) {}

    fn assert_resource_span_eq(&self, left: &ResourceSpans, right: &ResourceSpans) {
        assert_eq!(left.resource, right.resource);
        assert_eq!(left.schema_url, right.schema_url);

        assert_eq!(left.scope_spans.len(), right.scope_spans.len());
        let length = left.scope_spans.len();
        for i in 0..length {
            let left_spans = &left.scope_spans[i];
            let right_spans = &right.scope_spans[i];
            assert_eq!(left_spans.scope, right_spans.scope);
            assert_eq!(left_spans.schema_url, right_spans.schema_url);

            assert_eq!(left_spans.spans.len(), right_spans.spans.len());
            let length = left_spans.spans.len();
            for i in 0..length {
                let left_span = &left_spans.spans[i];
                let right_span = &right_spans.spans[i];
                assert_eq!(left_span.name, right_span.name);
                assert_eq!(left_span.trace_state, right_span.trace_state);
                assert_eq!(left_span.kind, right_span.kind);
                // ignore start_time_unit_nano
                // ignore end_time_unit_nano
                assert_eq!(left_span.attributes, right_span.attributes);
                assert_eq!(left_span.links, right_span.links);
                assert_eq!(left_span.status, right_span.status);

                assert_eq!(left_span.events.len(), right_span.events.len());
                let length = left_span.events.len();
                for i in 0..length {
                    let left_event = &left_span.events[i];
                    let right_event = &right_span.events[i];
                    assert_eq!(left_event.name, right_event.name);
                    // ignore time_unix_nano
                    assert_eq!(left_event.attributes, right_event.attributes);
                    assert_eq!(
                        left_event.dropped_attributes_count,
                        right_event.dropped_attributes_count
                    );
                }
            }
        }
    }

    fn assert_scope_span_eq(left: &ScopeSpans, right: &ScopeSpans) {
        assert_eq!(left.scope, right.scope);
        assert_eq!(left.schema_url, right.schema_url);
    }
}

// list of root spans
pub struct SpanForest {
    spans: HashMap<Vec<u8>, SpanTreeNode>,
}

impl SpanForest {
    pub fn from_spans(mut spans: Vec<Span>) -> Self {
        let mut forest = SpanForest {
            spans: HashMap::new(),
        };
        // We maintain a last seen spans set so that we can find their children
        let mut last_seen = HashSet::new();
        // first, identify all the root spans
        spans = spans
            .into_iter()
            .filter_map(|span| {
                if span.parent_span_id.is_empty() {
                    last_seen.insert(span.span_id.clone());
                    forest.add_root_span(span);
                    None
                } else {
                    Some(span)
                }
            })
            .collect();

        while !spans.is_empty() {
            let mut next_seen = HashSet::new();
            for span_id in last_seen {
                spans = spans
                    .into_iter()
                    .filter_map(|child_span| {
                        if child_span.parent_span_id == span_id {
                            next_seen.insert(child_span.span_id.clone());
                            forest
                                .spans
                                .get_mut(&span_id)
                                .unwrap()
                                .add_child(child_span);
                            None
                        } else {
                            Some(child_span)
                        }
                    })
                    .collect();
            }
            if next_seen.is_empty() {
                // when we didn't find any children, break
                break;
            }
            last_seen = next_seen;
        }

        if spans.len() > 0 {
            panic!("found spans with invalid parent: {:?}", spans);
        }

        forest
    }

    fn add_root_span(&mut self, span: Span) {
        let span_id = span.span_id.clone();
        let node = SpanTreeNode::new(span);
        self.spans.insert(span_id, node);
    }

    fn add_span(&mut self, span: Span) {
        if span.parent_span_id.is_empty() {
            self.add_root_span(span);
        } else {
        }
    }
}

// Compare span trees when their IDs are different
struct SpanTreeNode {
    span: Span,
    children: HashMap<Vec<u8>, SpanTreeNode>,
}

impl SpanTreeNode {
    fn new(span: Span) -> Self {
        SpanTreeNode {
            span,
            children: HashMap::new(),
        }
    }

    fn add_child(&mut self, child: Span) {
        self.children
            .insert(child.span_id.clone(), SpanTreeNode::new(child));
    }
}

impl PartialEq for SpanTreeNode {
    fn eq(&self, other: &Self) -> bool {
        span_eq(&self.span, &other.span) && self.children == other.children
    }
}

// assert all predicates parts of a span is the same
fn span_eq(left: &Span, right: &Span) -> bool {
    assert_eq!(left.name, right.name);
    assert_eq!(left.kind, right.kind);
    assert_eq!(left.trace_state, right.trace_state);
    // ignore start_time_unit_nano
    // ignore end_time_unit_nano
    assert_eq!(left.attributes, right.attributes);
    // assert_eq!(left.links, right.links);
    // todo: for link, we need to translate the span ids between `left` and `right`
    assert_eq!(left.status, right.status);

    assert_eq!(left.events.len(), right.events.len());
    let length = left.events.len();
    for i in 0..length {
        let left_event = &left.events[i];
        let right_event = &right.events[i];
        assert_eq!(left_event.name, right_event.name);
        // ignore time_unix_nano
        assert_eq!(left_event.attributes, right_event.attributes);
        assert_eq!(
            left_event.dropped_attributes_count,
            right_event.dropped_attributes_count
        );
    }
    true
}

pub fn read_spans_from_json(file: File) -> ResourceSpans {
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}