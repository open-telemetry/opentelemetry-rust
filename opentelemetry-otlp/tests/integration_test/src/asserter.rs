use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, Span, TracesData};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::fs::File;

// Given two ResourceSpans, assert that they are equal except for the timestamps
pub struct TraceAsserter {
    results: Vec<ResourceSpans>,
    expected: Vec<ResourceSpans>,
}

impl TraceAsserter {
    // Create a new TraceAsserter
    pub fn new(results: Vec<ResourceSpans>, expected: Vec<ResourceSpans>) -> Self {
        TraceAsserter { results, expected }
    }

    pub fn assert(self) {
        self.assert_resource_span_eq(&self.results, &self.expected);
    }

    fn assert_resource_span_eq(&self, results: &[ResourceSpans], expected: &[ResourceSpans]) {
        let mut results_spans = Vec::new();
        let mut expected_spans = Vec::new();

        assert_eq!(results.len(), expected.len());
        for i in 0..results.len() {
            let result_resource_span = &results[i];
            let expected_resource_span = &expected[i];
            assert_eq!(
                result_resource_span.resource,
                expected_resource_span.resource
            );
            assert_eq!(
                result_resource_span.schema_url,
                expected_resource_span.schema_url
            );

            assert_eq!(
                result_resource_span.scope_spans.len(),
                expected_resource_span.scope_spans.len()
            );

            for i in 0..result_resource_span.scope_spans.len() {
                let results_scope_span = &result_resource_span.scope_spans[i];
                let expected_results_span = &expected_resource_span.scope_spans[i];

                results_spans.extend(results_scope_span.spans.clone());
                expected_spans.extend(expected_results_span.spans.clone());
            }
        }

        let results_span_forest = SpanForest::from_spans(results_spans);
        let expected_span_forest = SpanForest::from_spans(expected_spans);
        assert_eq!(results_span_forest, expected_span_forest);
    }
}

// list of root spans
pub struct SpanForest {
    spans: HashMap<Vec<u8>, SpanTreeNode>,
}

impl Debug for SpanForest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpanForest")
            .field("spans", &self.spans)
            .finish()
    }
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

        if !spans.is_empty() {
            panic!("found spans with invalid parent: {:?}", spans);
        }

        forest
    }

    fn add_root_span(&mut self, span: Span) {
        let span_id = span.span_id.clone();
        let node = SpanTreeNode::new(span);
        self.spans.insert(span_id, node);
    }

    fn get_root_spans(&self) -> Vec<&SpanTreeNode> {
        self.spans
            .iter()
            .filter_map(|(_, span_node)| {
                if span_node.span.parent_span_id.is_empty() {
                    Some(span_node)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl PartialEq for SpanForest {
    fn eq(&self, other: &Self) -> bool {
        self.get_root_spans() == other.get_root_spans()
    }
}

// Compare span trees when their IDs are different
struct SpanTreeNode {
    span: Span,
    children: Vec<SpanTreeNode>,
}

impl Debug for SpanTreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpanTreeNode")
            .field("span", &self.span)
            .field("children", &self.children)
            .finish()
    }
}

impl SpanTreeNode {
    fn new(span: Span) -> Self {
        SpanTreeNode {
            span,
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, child: Span) {
        self.children.push(SpanTreeNode::new(child));
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

// read a file contains ResourceSpans in json format
pub fn read_spans_from_json(file: File) -> Vec<ResourceSpans> {
    let reader = std::io::BufReader::new(file);

    let trace_data: TracesData = serde_json::from_reader(reader).unwrap();
    trace_data.resource_spans
}
