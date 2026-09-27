#[cfg(feature = "gen-tonic-messages")]
/// Builds span flags based on the parent span's remote property.
/// This follows the OTLP specification for span flags.
pub(crate) fn build_span_flags(parent_span_is_remote: bool, base_flags: u32) -> u32 {
    use crate::proto::tonic::trace::v1::SpanFlags;
    let mut flags = base_flags;
    flags |= SpanFlags::ContextHasIsRemoteMask as u32;
    if parent_span_is_remote {
        flags |= SpanFlags::ContextIsRemoteMask as u32;
    }
    flags
}

#[cfg(feature = "gen-tonic-messages")]
pub mod tonic {
    use crate::proto::tonic::resource::v1::Resource;
    use crate::proto::tonic::trace::v1::{span, status, ResourceSpans, ScopeSpans, Span, Status};
    use crate::transform::common::{
        to_nanos,
        tonic::{Attributes, ResourceAttributesWithSchema},
    };
    use opentelemetry::trace;
    use opentelemetry::trace::{Link, SpanId, SpanKind};
    use opentelemetry_sdk::trace::SpanData;
    use std::collections::HashMap;

    impl From<SpanKind> for span::SpanKind {
        fn from(span_kind: SpanKind) -> Self {
            match span_kind {
                SpanKind::Client => span::SpanKind::Client,
                SpanKind::Consumer => span::SpanKind::Consumer,
                SpanKind::Internal => span::SpanKind::Internal,
                SpanKind::Producer => span::SpanKind::Producer,
                SpanKind::Server => span::SpanKind::Server,
            }
        }
    }

    impl From<&trace::Status> for status::StatusCode {
        fn from(status: &trace::Status) -> Self {
            match status {
                trace::Status::Ok => status::StatusCode::Ok,
                trace::Status::Unset => status::StatusCode::Unset,
                trace::Status::Error { .. } => status::StatusCode::Error,
            }
        }
    }

    impl From<Link> for span::Link {
        fn from(link: Link) -> Self {
            span::Link {
                trace_id: link.span_context.trace_id().to_bytes().to_vec(),
                span_id: link.span_context.span_id().to_bytes().to_vec(),
                trace_state: link.span_context.trace_state().header(),
                attributes: Attributes::from(link.attributes).0,
                dropped_attributes_count: link.dropped_attributes_count,
                flags: super::build_span_flags(
                    link.span_context.is_remote(),
                    link.span_context.trace_flags().to_u8() as u32,
                ),
            }
        }
    }
    impl From<opentelemetry_sdk::trace::SpanData> for Span {
        fn from(source_span: opentelemetry_sdk::trace::SpanData) -> Self {
            let span_kind: span::SpanKind = source_span.span_kind.into();
            Span {
                trace_id: source_span.span_context.trace_id().to_bytes().to_vec(),
                span_id: source_span.span_context.span_id().to_bytes().to_vec(),
                trace_state: source_span.span_context.trace_state().header(),
                parent_span_id: {
                    if source_span.parent_span_id != SpanId::INVALID {
                        source_span.parent_span_id.to_bytes().to_vec()
                    } else {
                        vec![]
                    }
                },
                flags: super::build_span_flags(
                    source_span.parent_span_is_remote,
                    source_span.span_context.trace_flags().to_u8() as u32,
                ),
                name: source_span.name.into_owned(),
                kind: span_kind as i32,
                start_time_unix_nano: to_nanos(source_span.start_time),
                end_time_unix_nano: to_nanos(source_span.end_time),
                dropped_attributes_count: source_span.dropped_attributes_count,
                attributes: Attributes::from(source_span.attributes).0,
                dropped_events_count: source_span.events.dropped_count,
                events: source_span
                    .events
                    .into_iter()
                    .map(|event| span::Event {
                        time_unix_nano: to_nanos(event.timestamp),
                        name: event.name.into(),
                        attributes: Attributes::from(event.attributes).0,
                        dropped_attributes_count: event.dropped_attributes_count,
                    })
                    .collect(),
                dropped_links_count: source_span.links.dropped_count,
                links: source_span.links.into_iter().map(Into::into).collect(),
                status: Some(Status {
                    code: status::StatusCode::from(&source_span.status).into(),
                    message: match source_span.status {
                        trace::Status::Error { description } => description.to_string(),
                        _ => Default::default(),
                    },
                }),
            }
        }
    }

    impl ResourceSpans {
        pub fn new(source_span: SpanData, resource: &ResourceAttributesWithSchema) -> Self {
            let span_kind: span::SpanKind = source_span.span_kind.into();
            ResourceSpans {
                resource: Some(Resource {
                    attributes: resource.attributes.0.clone(),
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                schema_url: resource.schema_url.clone().unwrap_or_default(),
                scope_spans: vec![ScopeSpans {
                    schema_url: source_span
                        .instrumentation_scope
                        .schema_url()
                        .map(ToOwned::to_owned)
                        .unwrap_or_default(),
                    scope: Some((source_span.instrumentation_scope, None).into()),
                    spans: vec![Span {
                        trace_id: source_span.span_context.trace_id().to_bytes().to_vec(),
                        span_id: source_span.span_context.span_id().to_bytes().to_vec(),
                        trace_state: source_span.span_context.trace_state().header(),
                        parent_span_id: {
                            if source_span.parent_span_id != SpanId::INVALID {
                                source_span.parent_span_id.to_bytes().to_vec()
                            } else {
                                vec![]
                            }
                        },
                        flags: super::build_span_flags(
                            source_span.parent_span_is_remote,
                            source_span.span_context.trace_flags().to_u8() as u32,
                        ),
                        name: source_span.name.into_owned(),
                        kind: span_kind as i32,
                        start_time_unix_nano: to_nanos(source_span.start_time),
                        end_time_unix_nano: to_nanos(source_span.end_time),
                        dropped_attributes_count: source_span.dropped_attributes_count,
                        attributes: Attributes::from(source_span.attributes).0,
                        dropped_events_count: source_span.events.dropped_count,
                        events: source_span
                            .events
                            .into_iter()
                            .map(|event| span::Event {
                                time_unix_nano: to_nanos(event.timestamp),
                                name: event.name.into(),
                                attributes: Attributes::from(event.attributes).0,
                                dropped_attributes_count: event.dropped_attributes_count,
                            })
                            .collect(),
                        dropped_links_count: source_span.links.dropped_count,
                        links: source_span.links.into_iter().map(Into::into).collect(),
                        status: Some(Status {
                            code: status::StatusCode::from(&source_span.status).into(),
                            message: match source_span.status {
                                trace::Status::Error { description } => description.to_string(),
                                _ => Default::default(),
                            },
                        }),
                    }],
                }],
            }
        }
    }

    pub fn group_spans_by_resource_and_scope(
        spans: Vec<SpanData>,
        resource: &ResourceAttributesWithSchema,
    ) -> Vec<ResourceSpans> {
        // Group spans by their instrumentation scope, converting each span as it
        // is consumed. The batch is owned, so spans are moved rather than cloned.
        let mut scope_indices: HashMap<opentelemetry::InstrumentationScope, usize> = HashMap::new();
        let mut scope_spans: Vec<ScopeSpans> = Vec::new();

        for span in spans {
            let scope_index = match scope_indices.get(&span.instrumentation_scope) {
                Some(&scope_index) => scope_index,
                None => {
                    let instrumentation = span.instrumentation_scope.clone();
                    let scope_index = scope_spans.len();
                    scope_spans.push(ScopeSpans {
                        scope: Some((&instrumentation, None).into()),
                        schema_url: instrumentation
                            .schema_url()
                            .map(ToOwned::to_owned)
                            .unwrap_or_default(),
                        spans: Vec::new(),
                    });
                    scope_indices.insert(instrumentation, scope_index);
                    scope_index
                }
            };

            scope_spans[scope_index].spans.push(span.into());
        }

        // Wrap ScopeSpans into a single ResourceSpans
        vec![ResourceSpans {
            resource: Some(Resource {
                attributes: resource.attributes.0.clone(),
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_spans,
            schema_url: resource.schema_url.clone().unwrap_or_default(),
        }]
    }
}

#[cfg(all(test, feature = "gen-tonic-messages"))]
mod span_flags_tests {
    use crate::proto::tonic::trace::v1::{Span, SpanFlags};
    use opentelemetry::trace::{SpanContext, SpanId, TraceFlags, TraceId, TraceState};
    use opentelemetry::InstrumentationScope;
    use opentelemetry_sdk::trace::SpanData;
    use std::borrow::Cow;

    #[test]
    fn test_build_span_flags_local_parent() {
        let flags = super::build_span_flags(false, 0); // is_remote = false
        assert_eq!(flags, SpanFlags::ContextHasIsRemoteMask as u32); // 0x100
    }

    #[test]
    fn test_build_span_flags_remote_parent() {
        let flags = super::build_span_flags(true, 0); // is_remote = true
        assert_eq!(
            flags,
            (SpanFlags::ContextHasIsRemoteMask as u32) | (SpanFlags::ContextIsRemoteMask as u32)
        ); // 0x300
    }

    #[test]
    fn test_build_span_flags_no_parent() {
        let flags = super::build_span_flags(false, 0); // no parent = false
        assert_eq!(flags, SpanFlags::ContextHasIsRemoteMask as u32); // 0x100
    }

    #[test]
    fn test_build_span_flags_preserves_base_flags() {
        let flags = super::build_span_flags(false, 0x01); // SAMPLED flag, local parent
        assert_eq!(flags, 0x01 | (SpanFlags::ContextHasIsRemoteMask as u32)); // 0x101
    }

    #[test]
    fn test_span_transformation_with_flags() {
        let span_data = SpanData {
            span_context: SpanContext::new(
                TraceId::from(789),
                SpanId::from(101112),
                TraceFlags::default(),
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::from(456),
            parent_span_is_remote: false,
            span_kind: opentelemetry::trace::SpanKind::Internal,
            name: Cow::Borrowed("test_span"),
            start_time: std::time::SystemTime::now(),
            end_time: std::time::SystemTime::now(),
            attributes: vec![],
            dropped_attributes_count: 0,
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status: opentelemetry::trace::Status::Unset,
            instrumentation_scope: InstrumentationScope::builder("test").build(),
        };

        let otlp_span: Span = span_data.into();
        assert_eq!(otlp_span.flags, SpanFlags::ContextHasIsRemoteMask as u32); // 0x100
    }

    #[test]
    fn test_span_transformation_with_remote_parent() {
        let span_data = SpanData {
            span_context: SpanContext::new(
                TraceId::from(789),
                SpanId::from(101112),
                TraceFlags::default(),
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::from(456),
            parent_span_is_remote: true,
            span_kind: opentelemetry::trace::SpanKind::Internal,
            name: Cow::Borrowed("test_span"),
            start_time: std::time::SystemTime::now(),
            end_time: std::time::SystemTime::now(),
            attributes: vec![],
            dropped_attributes_count: 0,
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status: opentelemetry::trace::Status::Unset,
            instrumentation_scope: InstrumentationScope::builder("test").build(),
        };

        let otlp_span: Span = span_data.into();
        assert_eq!(
            otlp_span.flags,
            (SpanFlags::ContextHasIsRemoteMask as u32) | (SpanFlags::ContextIsRemoteMask as u32)
        ); // 0x300
    }
}

#[cfg(test)]
mod tests {
    use crate::tonic::common::v1::{
        any_value::Value, AnyValue, InstrumentationScope as ProtoScope, KeyValue as ProtoKeyValue,
    };
    use crate::tonic::resource::v1::Resource as ProtoResource;
    use crate::tonic::trace::v1::{
        span, status, ResourceSpans, ScopeSpans, Span, Status as ProtoStatus,
    };
    use crate::transform::common::tonic::ResourceAttributesWithSchema;
    use opentelemetry::time::now;
    use opentelemetry::trace::{
        Event, Link, SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
    };
    use opentelemetry::InstrumentationScope;
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::resource::Resource;
    use opentelemetry_sdk::trace::{SpanData, SpanEvents, SpanLinks};
    use std::borrow::Cow;
    use std::time::{Duration, UNIX_EPOCH};

    fn proto_attribute(key: &str, value: Value) -> ProtoKeyValue {
        ProtoKeyValue {
            key: key.to_owned(),
            value: Some(AnyValue { value: Some(value) }),
            key_strindex: 0,
        }
    }

    #[test]
    fn span_data_transforms_to_expected_otlp_contract() {
        let start_time = UNIX_EPOCH + Duration::from_nanos(1_000);
        let end_time = UNIX_EPOCH + Duration::from_nanos(2_500);
        let scope = InstrumentationScope::builder("contract-lib")
            .with_version("2.1.0")
            .with_schema_url("https://example.com/scope/1")
            .with_attributes([KeyValue::new("scope.attr", true)])
            .build();
        let span_data = SpanData {
            span_context: SpanContext::new(
                TraceId::from(0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00),
                SpanId::from(0x1020_3040_5060_7080),
                TraceFlags::SAMPLED,
                false,
                "vendor=span".parse().unwrap(),
            ),
            parent_span_id: SpanId::from(0x8877_6655_4433_2211),
            parent_span_is_remote: true,
            span_kind: SpanKind::Server,
            name: Cow::Borrowed("contract span"),
            start_time,
            end_time,
            attributes: vec![
                KeyValue::new("str", "value"),
                KeyValue::new("bool", true),
                KeyValue::new("int", 42_i64),
                KeyValue::new("float", 3.5_f64),
            ],
            dropped_attributes_count: 2,
            events: {
                let mut events = SpanEvents::default();
                events.events = vec![Event::new(
                    "exception",
                    UNIX_EPOCH + Duration::from_nanos(1_500),
                    vec![KeyValue::new("handled", false)],
                    3,
                )];
                events.dropped_count = 4;
                events
            },
            links: {
                let mut links = SpanLinks::default();
                links.links = vec![Link::new(
                    SpanContext::new(
                        TraceId::from(0x2233_4455_6677_8899_aabb_ccdd_eeff_0011),
                        SpanId::from(0x2030_4050_6070_8090),
                        TraceFlags::SAMPLED,
                        true,
                        "vendor=link".parse().unwrap(),
                    ),
                    vec![KeyValue::new("link.attr", "linked")],
                    5,
                )];
                links.dropped_count = 6;
                links
            },
            status: Status::Error {
                description: "failed".into(),
            },
            instrumentation_scope: scope,
        };
        let resource = Resource::builder_empty()
            .with_schema_url(
                [KeyValue::new("service.name", "checkout")],
                "https://example.com/resource/1",
            )
            .build();
        let resource: ResourceAttributesWithSchema = (&resource).into();

        let actual = crate::transform::trace::tonic::group_spans_by_resource_and_scope(
            vec![span_data],
            &resource,
        );
        let expected = vec![ResourceSpans {
            resource: Some(ProtoResource {
                attributes: vec![proto_attribute(
                    "service.name",
                    Value::StringValue("checkout".to_owned()),
                )],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            schema_url: "https://example.com/resource/1".to_owned(),
            scope_spans: vec![ScopeSpans {
                scope: Some(ProtoScope {
                    name: "contract-lib".to_owned(),
                    version: "2.1.0".to_owned(),
                    attributes: vec![proto_attribute("scope.attr", Value::BoolValue(true))],
                    dropped_attributes_count: 0,
                }),
                schema_url: "https://example.com/scope/1".to_owned(),
                spans: vec![Span {
                    trace_id: 0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00_u128
                        .to_be_bytes()
                        .to_vec(),
                    span_id: 0x1020_3040_5060_7080_u64.to_be_bytes().to_vec(),
                    trace_state: "vendor=span".to_owned(),
                    parent_span_id: 0x8877_6655_4433_2211_u64.to_be_bytes().to_vec(),
                    flags: 0x301, // Sampled, with a known remote parent.
                    name: "contract span".to_owned(),
                    kind: span::SpanKind::Server as i32,
                    start_time_unix_nano: 1_000,
                    end_time_unix_nano: 2_500,
                    attributes: vec![
                        proto_attribute("str", Value::StringValue("value".to_owned())),
                        proto_attribute("bool", Value::BoolValue(true)),
                        proto_attribute("int", Value::IntValue(42)),
                        proto_attribute("float", Value::DoubleValue(3.5)),
                    ],
                    dropped_attributes_count: 2,
                    events: vec![span::Event {
                        time_unix_nano: 1_500,
                        name: "exception".to_owned(),
                        attributes: vec![proto_attribute("handled", Value::BoolValue(false))],
                        dropped_attributes_count: 3,
                    }],
                    dropped_events_count: 4,
                    links: vec![span::Link {
                        trace_id: 0x2233_4455_6677_8899_aabb_ccdd_eeff_0011_u128
                            .to_be_bytes()
                            .to_vec(),
                        span_id: 0x2030_4050_6070_8090_u64.to_be_bytes().to_vec(),
                        trace_state: "vendor=link".to_owned(),
                        attributes: vec![proto_attribute(
                            "link.attr",
                            Value::StringValue("linked".to_owned()),
                        )],
                        dropped_attributes_count: 5,
                        flags: 0x301,
                    }],
                    dropped_links_count: 6,
                    status: Some(ProtoStatus {
                        code: status::StatusCode::Error as i32,
                        message: "failed".to_owned(),
                    }),
                }],
            }],
        }];

        assert_eq!(actual, expected);
    }

    #[test]
    fn span_batch_preserves_scope_identity_and_span_associations() {
        let scope = |version, schema_url, enabled| {
            InstrumentationScope::builder("contract-lib")
                .with_version(version)
                .with_schema_url(schema_url)
                .with_attributes([KeyValue::new("enabled", enabled)])
                .build()
        };
        let make_span = |id, instrumentation_scope| SpanData {
            span_context: SpanContext::new(
                TraceId::from_bytes([1; 16]),
                SpanId::from(id),
                TraceFlags::default(),
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::INVALID,
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: "batch span".into(),
            start_time: UNIX_EPOCH + Duration::from_nanos(1_000),
            end_time: UNIX_EPOCH + Duration::from_nanos(2_500),
            attributes: vec![],
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope,
        };
        // Each metadata variant differs from the base scope in exactly one field.
        let batch = vec![
            make_span(1, scope("1.0", "https://example.com/scope/1", true)),
            make_span(2, scope("2.0", "https://example.com/scope/1", true)),
            make_span(3, scope("1.0", "https://example.com/scope/1", true)),
            make_span(4, scope("1.0", "https://example.com/scope/2", true)),
            make_span(5, scope("1.0", "https://example.com/scope/1", false)),
            make_span(6, scope("2.0", "https://example.com/scope/1", true)),
        ];
        let resource = Resource::builder_empty()
            .with_schema_url(
                [KeyValue::new("service.name", "batch-service")],
                "https://example.com/resource/1",
            )
            .build();
        let resource: ResourceAttributesWithSchema = (&resource).into();
        let mut actual =
            crate::transform::trace::tonic::group_spans_by_resource_and_scope(batch, &resource);

        let expected_span = |id: u64| Span {
            trace_id: vec![1; 16],
            span_id: id.to_be_bytes().to_vec(),
            trace_state: String::new(),
            parent_span_id: vec![],
            flags: 0x100, // Unsampled, with a known non-remote parent state.
            name: "batch span".to_owned(),
            kind: span::SpanKind::Internal as i32,
            start_time_unix_nano: 1_000,
            end_time_unix_nano: 2_500,
            attributes: vec![],
            dropped_attributes_count: 0,
            events: vec![],
            dropped_events_count: 0,
            links: vec![],
            dropped_links_count: 0,
            status: Some(ProtoStatus {
                code: status::StatusCode::Unset as i32,
                message: String::new(),
            }),
        };
        let expected_scope = |version: &str, schema_url: &str, enabled, spans| ScopeSpans {
            scope: Some(ProtoScope {
                name: "contract-lib".to_owned(),
                version: version.to_owned(),
                attributes: vec![proto_attribute("enabled", Value::BoolValue(enabled))],
                dropped_attributes_count: 0,
            }),
            schema_url: schema_url.to_owned(),
            spans,
        };
        let expected = vec![ResourceSpans {
            resource: Some(ProtoResource {
                attributes: vec![proto_attribute(
                    "service.name",
                    Value::StringValue("batch-service".to_owned()),
                )],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            schema_url: "https://example.com/resource/1".to_owned(),
            scope_spans: vec![
                expected_scope(
                    "1.0",
                    "https://example.com/scope/1",
                    true,
                    vec![expected_span(1), expected_span(3)],
                ),
                expected_scope(
                    "2.0",
                    "https://example.com/scope/1",
                    true,
                    vec![expected_span(2), expected_span(6)],
                ),
                expected_scope(
                    "1.0",
                    "https://example.com/scope/2",
                    true,
                    vec![expected_span(4)],
                ),
                expected_scope(
                    "1.0",
                    "https://example.com/scope/1",
                    false,
                    vec![expected_span(5)],
                ),
            ],
        }];

        // OTLP does not require scope groups or spans to follow the input order.
        for resource_spans in &mut actual {
            for scope_spans in &mut resource_spans.scope_spans {
                scope_spans
                    .spans
                    .sort_unstable_by(|a, b| a.span_id.cmp(&b.span_id));
            }
            resource_spans.scope_spans.sort_unstable_by(|a, b| {
                a.spans
                    .first()
                    .map(|s| &s.span_id)
                    .cmp(&b.spans.first().map(|s| &s.span_id))
            });
        }
        assert_eq!(actual, expected);
    }

    fn create_test_span_data(instrumentation_name: &'static str) -> SpanData {
        let span_context = SpanContext::new(
            TraceId::from(123),
            SpanId::from(456),
            TraceFlags::default(),
            false,
            TraceState::default(),
        );

        SpanData {
            span_context,
            parent_span_id: SpanId::from(0),
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: Cow::Borrowed("test_span"),
            start_time: now(),
            end_time: now() + Duration::from_secs(1),
            attributes: vec![KeyValue::new("key", "value")],
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope: InstrumentationScope::builder(instrumentation_name).build(),
        }
    }

    #[test]
    fn test_group_spans_by_resource_and_scope_single_scope() {
        let resource = Resource::builder_empty()
            .with_attribute(KeyValue::new("resource_key", "resource_value"))
            .build();
        let span_data = create_test_span_data("lib1");

        let spans = vec![span_data.clone()];
        let resource: ResourceAttributesWithSchema = (&resource).into(); // Convert Resource to ResourceAttributesWithSchema

        let grouped_spans =
            crate::transform::trace::tonic::group_spans_by_resource_and_scope(spans, &resource);

        assert_eq!(grouped_spans.len(), 1);

        let resource_spans = &grouped_spans[0];
        assert_eq!(
            resource_spans.resource.as_ref().unwrap().attributes.len(),
            1
        );
        assert_eq!(
            resource_spans.resource.as_ref().unwrap().attributes[0].key,
            "resource_key"
        );
        assert_eq!(
            resource_spans.resource.as_ref().unwrap().attributes[0]
                .value
                .clone()
                .unwrap()
                .value
                .unwrap(),
            Value::StringValue("resource_value".to_string())
        );

        let scope_spans = &resource_spans.scope_spans;
        assert_eq!(scope_spans.len(), 1);

        let scope_span = &scope_spans[0];
        assert_eq!(scope_span.scope.as_ref().unwrap().name, "lib1");
        assert_eq!(scope_span.spans.len(), 1);

        assert_eq!(
            scope_span.spans[0].trace_id,
            span_data.span_context.trace_id().to_bytes().to_vec()
        );
    }

    #[test]
    fn test_group_spans_by_resource_and_scope_multiple_scopes() {
        let resource = Resource::builder_empty()
            .with_attribute(KeyValue::new("resource_key", "resource_value"))
            .build();
        let span_data1 = create_test_span_data("lib1");
        let span_data2 = create_test_span_data("lib1");
        let span_data3 = create_test_span_data("lib2");

        let spans = vec![span_data1.clone(), span_data2.clone(), span_data3.clone()];
        let resource: ResourceAttributesWithSchema = (&resource).into(); // Convert Resource to ResourceAttributesWithSchema

        let grouped_spans =
            crate::transform::trace::tonic::group_spans_by_resource_and_scope(spans, &resource);

        assert_eq!(grouped_spans.len(), 1);

        let resource_spans = &grouped_spans[0];
        assert_eq!(
            resource_spans.resource.as_ref().unwrap().attributes.len(),
            1
        );
        assert_eq!(
            resource_spans.resource.as_ref().unwrap().attributes[0].key,
            "resource_key"
        );
        assert_eq!(
            resource_spans.resource.as_ref().unwrap().attributes[0]
                .value
                .clone()
                .unwrap()
                .value
                .unwrap(),
            Value::StringValue("resource_value".to_string())
        );

        let scope_spans = &resource_spans.scope_spans;
        assert_eq!(scope_spans.len(), 2);

        // Check the scope spans for both lib1 and lib2
        let mut lib1_scope_span = None;
        let mut lib2_scope_span = None;

        for scope_span in scope_spans {
            match scope_span.scope.as_ref().unwrap().name.as_str() {
                "lib1" => lib1_scope_span = Some(scope_span),
                "lib2" => lib2_scope_span = Some(scope_span),
                _ => {}
            }
        }

        let lib1_scope_span = lib1_scope_span.expect("lib1 scope span not found");
        let lib2_scope_span = lib2_scope_span.expect("lib2 scope span not found");

        assert_eq!(lib1_scope_span.scope.as_ref().unwrap().name, "lib1");
        assert_eq!(lib2_scope_span.scope.as_ref().unwrap().name, "lib2");

        assert_eq!(lib1_scope_span.spans.len(), 2);
        assert_eq!(lib2_scope_span.spans.len(), 1);

        assert_eq!(
            lib1_scope_span.spans[0].trace_id,
            span_data1.span_context.trace_id().to_bytes().to_vec()
        );
        assert_eq!(
            lib1_scope_span.spans[1].trace_id,
            span_data2.span_context.trace_id().to_bytes().to_vec()
        );
        assert_eq!(
            lib2_scope_span.spans[0].trace_id,
            span_data3.span_context.trace_id().to_bytes().to_vec()
        );
    }

    #[test]
    fn test_scope_spans_uses_instrumentation_schema_url_not_resource() {
        let resource = Resource::builder_empty()
            .with_schema_url(vec![], "http://resource-schema")
            .build();

        let instrumentation_scope = InstrumentationScope::builder("test-lib")
            .with_schema_url("http://instrumentation-schema")
            .build();

        let span_data = SpanData {
            span_context: SpanContext::new(
                TraceId::from(123),
                SpanId::from(456),
                TraceFlags::default(),
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::from(0),
            parent_span_is_remote: false,
            span_kind: SpanKind::Internal,
            name: Cow::Borrowed("test_span"),
            start_time: now(),
            end_time: now() + Duration::from_secs(1),
            attributes: vec![],
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope,
        };

        let resource: ResourceAttributesWithSchema = (&resource).into();
        let grouped_spans = crate::transform::trace::tonic::group_spans_by_resource_and_scope(
            vec![span_data],
            &resource,
        );

        assert_eq!(grouped_spans.len(), 1);
        let resource_spans = &grouped_spans[0];
        assert_eq!(resource_spans.schema_url, "http://resource-schema");

        let scope_spans = &resource_spans.scope_spans;
        assert_eq!(scope_spans.len(), 1);
        assert_eq!(scope_spans[0].schema_url, "http://instrumentation-schema");
    }
}
