#[cfg(all(feature = "with-serde", feature = "gen-tonic-messages"))]
mod json_serde {
    #[cfg(feature = "logs")]
    use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    #[cfg(feature = "metrics")]
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    #[cfg(feature = "trace")]
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::common::v1::any_value::Value;
    use opentelemetry_proto::tonic::common::v1::{
        AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList,
    };
    #[cfg(feature = "logs")]
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    #[cfg(feature = "metrics")]
    use opentelemetry_proto::tonic::metrics::v1::{
        exemplar::Value as ExemplarValue, metric::Data, number_data_point::Value as MetricValue,
        ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint, Metric,
        NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum,
    };
    use opentelemetry_proto::tonic::resource::v1::Resource;
    #[cfg(feature = "trace")]
    use opentelemetry_proto::tonic::trace::v1::{
        span::{Event, Link},
        ResourceSpans, ScopeSpans, Span, Status,
    };

    #[cfg(feature = "trace")]
    mod export_trace_service_request {
        use super::*;

        // `ExportTraceServiceRequest` from the OpenTelemetry proto examples
        // see <https://github.com/open-telemetry/opentelemetry-proto/blob/v1.3.2/examples/trace.json>
        mod example {
            use super::*;

            fn value() -> ExportTraceServiceRequest {
                ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: String::from("service.name"),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue(String::from("my.service"))),
                                }),
                                key_strindex: 0,
                            }],
                            dropped_attributes_count: 0,
                            entity_refs: vec![],
                        }),
                        scope_spans: vec![ScopeSpans {
                            scope: Some(InstrumentationScope {
                                name: String::from("my.library"),
                                version: String::from("1.0.0"),
                                attributes: vec![KeyValue {
                                    key: String::from("my.scope.attribute"),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue(String::from(
                                            "some scope attribute",
                                        ))),
                                    }),
                                    key_strindex: 0,
                                }],
                                dropped_attributes_count: 0,
                            }),
                            spans: vec![Span {
                                trace_id: const_hex::decode("5b8efff798038103d269b633813fc60c")
                                    .unwrap(),
                                span_id: const_hex::decode("eee19b7ec3c1b174").unwrap(),
                                trace_state: String::new(),
                                parent_span_id: const_hex::decode("eee19b7ec3c1b173").unwrap(),
                                flags: 0,
                                name: String::from("I'm a server span"),
                                kind: 2,
                                start_time_unix_nano: 1544712660000000000,
                                end_time_unix_nano: 1544712661000000000,
                                attributes: vec![KeyValue {
                                    key: String::from("my.span.attr"),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue(String::from("some value"))),
                                    }),
                                    key_strindex: 0,
                                }],
                                dropped_attributes_count: 0,
                                events: vec![],
                                dropped_events_count: 0,
                                links: vec![],
                                dropped_links_count: 0,
                                status: None,
                            }],
                            schema_url: String::new(),
                        }],
                        schema_url: String::new(),
                    }],
                }
            }

            // language=json
            const CANONICAL: &str = r#"{
  "resourceSpans": [
    {
      "resource": {
        "attributes": [
          {
            "key": "service.name",
            "value": {
              "stringValue": "my.service"
            }
          }
        ],
        "droppedAttributesCount": 0,
        "entityRefs": []
      },
      "scopeSpans": [
        {
          "scope": {
            "name": "my.library",
            "version": "1.0.0",
            "attributes": [
              {
                "key": "my.scope.attribute",
                "value": {
                  "stringValue": "some scope attribute"
                }
              }
            ],
            "droppedAttributesCount": 0
          },
          "spans": [
            {
              "traceId": "5b8efff798038103d269b633813fc60c",
              "spanId": "eee19b7ec3c1b174",
              "traceState": "",
              "parentSpanId": "eee19b7ec3c1b173",
              "flags": 0,
              "name": "I'm a server span",
              "kind": 2,
              "startTimeUnixNano": "1544712660000000000",
              "endTimeUnixNano": "1544712661000000000",
              "attributes": [
                {
                  "key": "my.span.attr",
                  "value": {
                    "stringValue": "some value"
                  }
                }
              ],
              "droppedAttributesCount": 0,
              "events": [],
              "droppedEventsCount": 0,
              "links": [],
              "droppedLinksCount": 0,
              "status": null
            }
          ],
          "schemaUrl": ""
        }
      ],
      "schemaUrl": ""
    }
  ]
}"#;

            // copied from the example json file
            // language=json
            const ALTERNATIVE: &str = r#"{
  "resourceSpans": [
    {
      "resource": {
        "attributes": [
          {
            "key": "service.name",
            "value": {
              "stringValue": "my.service"
            }
          }
        ]
      },
      "scopeSpans": [
        {
          "scope": {
            "name": "my.library",
            "version": "1.0.0",
            "attributes": [
              {
                "key": "my.scope.attribute",
                "value": {
                  "stringValue": "some scope attribute"
                }
              }
            ]
          },
          "spans": [
            {
              "traceId": "5B8EFFF798038103D269B633813FC60C",
              "spanId": "EEE19B7EC3C1B174",
              "parentSpanId": "EEE19B7EC3C1B173",
              "name": "I'm a server span",
              "startTimeUnixNano": "1544712660000000000",
              "endTimeUnixNano": "1544712661000000000",
              "kind": 2,
              "attributes": [
                {
                  "key": "my.span.attr",
                  "value": {
                    "stringValue": "some value"
                  }
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
"#;

            #[test]
            fn serialize() {
                let input: ExportTraceServiceRequest = value();
                let actual =
                    serde_json::to_string_pretty(&input).expect("serialization must succeed");
                assert_eq!(actual, CANONICAL);
            }

            #[test]
            fn deserialize_canonical() {
                let actual: ExportTraceServiceRequest =
                    serde_json::from_str(CANONICAL).expect("deserialization must succeed");
                let expected: ExportTraceServiceRequest = value();
                assert_eq!(actual, expected);
            }

            #[test]
            fn deserialize_alternative() {
                let actual: ExportTraceServiceRequest =
                    serde_json::from_str(ALTERNATIVE).expect("deserialization must succeed");
                let expected: ExportTraceServiceRequest = value();
                assert_eq!(actual, expected);
            }
        }

        // request where all fields are set
        mod complete {
            use super::*;

            fn value() -> ExportTraceServiceRequest {
                ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: String::from("service.name"),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue(String::from("my.service"))),
                                }),
                                key_strindex: 0,
                            }],
                            dropped_attributes_count: 1,
                            entity_refs: vec![],
                        }),
                        scope_spans: vec![ScopeSpans {
                            scope: Some(InstrumentationScope {
                                name: String::from("my.library"),
                                version: String::from("1.0.0"),
                                attributes: vec![KeyValue {
                                    key: String::from("my.scope.attribute"),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue(String::from(
                                            "some scope attribute",
                                        ))),
                                    }),
                                    key_strindex: 0,
                                }],
                                dropped_attributes_count: 1,
                            }),
                            spans: vec![Span {
                                trace_id: const_hex::decode("5b8efff798038103d269b633813fc60c")
                                    .unwrap(),
                                span_id: const_hex::decode("eee19b7ec3c1b174").unwrap(),
                                trace_state: String::from("browser=firefox,os=linux"),
                                parent_span_id: const_hex::decode("eee19b7ec3c1b173").unwrap(),
                                flags: 1,
                                name: String::from("I'm a server span"),
                                kind: 2,
                                start_time_unix_nano: 1544712660000000000,
                                end_time_unix_nano: 1544712661000000000,
                                attributes: vec![
                                    KeyValue {
                                        key: String::from("my.span.attr"),
                                        value: Some(AnyValue {
                                            value: Some(Value::StringValue(String::from(
                                                "some value",
                                            ))),
                                        }),
                                        key_strindex: 0,
                                    },
                                    KeyValue {
                                        key: String::from("my.span.bytes.attr"),
                                        value: Some(AnyValue {
                                            value: Some(Value::BytesValue(vec![0x80, 0x80, 0x80])),
                                        }),
                                        key_strindex: 0,
                                    },
                                ],
                                dropped_attributes_count: 1,
                                events: vec![Event {
                                    time_unix_nano: 1544712660500000000,
                                    name: String::from("user.created"),
                                    attributes: vec![KeyValue {
                                        key: String::from("my.event.attr"),
                                        value: Some(AnyValue {
                                            value: Some(Value::StringValue(String::from(
                                                "snowman",
                                            ))),
                                        }),
                                        key_strindex: 0,
                                    }],
                                    dropped_attributes_count: 1,
                                }],
                                dropped_events_count: 1,
                                links: vec![Link {
                                    trace_id: const_hex::decode("5b8efff798038103d269b633813fc60b")
                                        .unwrap(),
                                    span_id: const_hex::decode("eee19b7ec3c1b172").unwrap(),
                                    trace_state: String::from("food=pizza,color=red"),
                                    attributes: vec![KeyValue {
                                        key: String::from("my.link.attr"),
                                        value: Some(AnyValue {
                                            value: Some(Value::StringValue(String::from("rust"))),
                                        }),
                                        key_strindex: 0,
                                    }],
                                    dropped_attributes_count: 1,
                                    flags: 0x0200,
                                }],
                                dropped_links_count: 1,
                                status: Some(Status {
                                    message: String::from("service temporarily unavailable"),
                                    code: 2,
                                }),
                            }],
                            schema_url: String::from("https://opentelemetry.io/schemas/1.24.0"),
                        }],
                        schema_url: String::from("https://opentelemetry.io/schemas/1.24.0"),
                    }],
                }
            }

            // language=json
            const CANONICAL: &str = r#"{
  "resourceSpans": [
    {
      "resource": {
        "attributes": [
          {
            "key": "service.name",
            "value": {
              "stringValue": "my.service"
            }
          }
        ],
        "droppedAttributesCount": 1,
        "entityRefs": []
      },
      "scopeSpans": [
        {
          "scope": {
            "name": "my.library",
            "version": "1.0.0",
            "attributes": [
              {
                "key": "my.scope.attribute",
                "value": {
                  "stringValue": "some scope attribute"
                }
              }
            ],
            "droppedAttributesCount": 1
          },
          "spans": [
            {
              "traceId": "5b8efff798038103d269b633813fc60c",
              "spanId": "eee19b7ec3c1b174",
              "traceState": "browser=firefox,os=linux",
              "parentSpanId": "eee19b7ec3c1b173",
              "flags": 1,
              "name": "I'm a server span",
              "kind": 2,
              "startTimeUnixNano": "1544712660000000000",
              "endTimeUnixNano": "1544712661000000000",
              "attributes": [
                {
                  "key": "my.span.attr",
                  "value": {
                    "stringValue": "some value"
                  }
                },
                {
                  "key": "my.span.bytes.attr",
                  "value": {
                    "bytesValue": "gICA"
                  }
                }
              ],
              "droppedAttributesCount": 1,
              "events": [
                {
                  "timeUnixNano": "1544712660500000000",
                  "name": "user.created",
                  "attributes": [
                    {
                      "key": "my.event.attr",
                      "value": {
                        "stringValue": "snowman"
                      }
                    }
                  ],
                  "droppedAttributesCount": 1
                }
              ],
              "droppedEventsCount": 1,
              "links": [
                {
                  "traceId": "5b8efff798038103d269b633813fc60b",
                  "spanId": "eee19b7ec3c1b172",
                  "traceState": "food=pizza,color=red",
                  "attributes": [
                    {
                      "key": "my.link.attr",
                      "value": {
                        "stringValue": "rust"
                      }
                    }
                  ],
                  "droppedAttributesCount": 1,
                  "flags": 512
                }
              ],
              "droppedLinksCount": 1,
              "status": {
                "message": "service temporarily unavailable",
                "code": 2
              }
            }
          ],
          "schemaUrl": "https://opentelemetry.io/schemas/1.24.0"
        }
      ],
      "schemaUrl": "https://opentelemetry.io/schemas/1.24.0"
    }
  ]
}"#;

            #[test]
            fn serialize() {
                let input: ExportTraceServiceRequest = value();
                let actual =
                    serde_json::to_string_pretty(&input).expect("serialization must succeed");
                assert_eq!(actual, CANONICAL);
            }

            #[test]
            fn deserialize_canonical() {
                let actual: ExportTraceServiceRequest =
                    serde_json::from_str(CANONICAL).expect("deserialization must succeed");
                let expected: ExportTraceServiceRequest = value();
                assert_eq!(actual, expected);
            }
        }
    }

    mod value {
        use super::*;

        #[test]
        fn string() {
            let value = Value::StringValue(String::from("my.service"));
            // language=json
            let json = r#"{"stringValue":"my.service"}"#;
            assert_eq!(
                serde_json::to_string(&value).expect("serialization succeeds"),
                json
            );
            assert_eq!(
                serde_json::from_str::<Value>(json).expect("deserialization succeeds"),
                value
            );
        }

        #[test]
        fn bool() {
            let value = Value::BoolValue(true);
            // language=json
            let json = r#"{"boolValue":true}"#;
            assert_eq!(
                serde_json::to_string(&value).expect("serialization succeeds"),
                json
            );
            assert_eq!(
                serde_json::from_str::<Value>(json).expect("deserialization succeeds"),
                value
            );
        }

        #[test]
        fn int() {
            let value = Value::IntValue(123);
            // language=json
            let json = r#"{"intValue":123}"#;
            assert_eq!(
                serde_json::to_string(&value).expect("serialization succeeds"),
                json
            );
            assert_eq!(
                serde_json::from_str::<Value>(json).expect("deserialization succeeds"),
                value
            );
        }

        #[test]
        fn array_empty() {
            let value = Value::ArrayValue(ArrayValue { values: vec![] });
            // language=json
            let json = r#"{"arrayValue":{"values":[]}}"#;
            assert_eq!(
                serde_json::to_string(&value).expect("serialization succeeds"),
                json
            );
            assert_eq!(
                serde_json::from_str::<Value>(json).expect("deserialization succeeds"),
                value
            );
        }

        #[test]
        fn array_strings() {
            let value = Value::ArrayValue(ArrayValue {
                values: vec![
                    AnyValue {
                        value: Some(Value::StringValue(String::from("foo"))),
                    },
                    AnyValue {
                        value: Some(Value::StringValue(String::from("bar"))),
                    },
                ],
            });
            // language=json
            let json = r#"{
  "arrayValue": {
    "values": [
      {
        "stringValue": "foo"
      },
      {
        "stringValue": "bar"
      }
    ]
  }
}"#;
            assert_eq!(
                serde_json::to_string_pretty(&value).expect("serialization succeeds"),
                json
            );
            assert_eq!(
                serde_json::from_str::<Value>(json).expect("deserialization succeeds"),
                value
            );
        }

        #[test]
        fn array_mixed() {
            let value = Value::ArrayValue(ArrayValue {
                values: vec![
                    AnyValue {
                        value: Some(Value::StringValue(String::from("foo"))),
                    },
                    AnyValue {
                        value: Some(Value::IntValue(1337)),
                    },
                ],
            });
            // language=json
            let json = r#"{
  "arrayValue": {
    "values": [
      {
        "stringValue": "foo"
      },
      {
        "intValue": "1337"
      }
    ]
  }
}"#;
            assert_eq!(
                serde_json::to_string_pretty(&value).expect("serialization succeeds"),
                json
            );
            assert_eq!(
                serde_json::from_str::<Value>(json).expect("deserialization succeeds"),
                value
            );
        }

        #[test]
        fn map_single_string() {
            let value = Value::KvlistValue(KeyValueList {
                values: vec![KeyValue {
                    key: String::from("some.map.key"),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue(String::from("some value"))),
                    }),
                    key_strindex: 0,
                }],
            });
            // language=json
            let json = r#"{
  "kvlistValue": {
    "values": [
      {
        "key": "some.map.key",
        "value": {
          "stringValue": "some value"
        }
      }
    ]
  }
}"#;
            assert_eq!(
                serde_json::to_string_pretty(&value).expect("serialization succeeds"),
                json
            );
            assert_eq!(
                serde_json::from_str::<Value>(json).expect("deserialization succeeds"),
                value
            );
        }
    }

    mod key_value {
        use super::*;

        mod string {
            use super::*;

            fn value() -> KeyValue {
                KeyValue {
                    key: String::from("service.name"),
                    value: Some(AnyValue {
                        value: Some(Value::StringValue(String::from("my.service"))),
                    }),
                    key_strindex: 0,
                }
            }

            // language=json
            const CANONICAL: &str = r#"{
  "key": "service.name",
  "value": {
    "stringValue": "my.service"
  }
}"#;

            #[test]
            fn serialize() {
                let input: KeyValue = value();
                let actual =
                    serde_json::to_string_pretty(&input).expect("serialization must succeed");
                assert_eq!(actual, CANONICAL);
            }

            #[test]
            fn deserialize_canonical() {
                let actual: KeyValue =
                    serde_json::from_str(CANONICAL).expect("deserialization must succeed");
                let expected: KeyValue = value();
                assert_eq!(actual, expected);
            }
        }

        mod int {
            use super::*;

            fn value() -> KeyValue {
                KeyValue {
                    key: String::from("service.id"),
                    value: Some(AnyValue {
                        value: Some(Value::IntValue(33)),
                    }),
                    key_strindex: 0,
                }
            }

            // language=json
            const CANONICAL: &str = r#"{
  "key": "service.id",
  "value": {
    "intValue": "33"
  }
}"#;

            // unquoted int value
            // language=json
            const ALTERNATIVE: &str = r#"{
  "key": "service.id",
  "value": {
    "intValue": 33
  }
}"#;

            #[test]
            fn serialize() {
                let input: KeyValue = value();
                let actual =
                    serde_json::to_string_pretty(&input).expect("serialization must succeed");
                assert_eq!(actual, CANONICAL);
            }

            #[test]
            fn deserialize_canonical() {
                let actual: KeyValue =
                    serde_json::from_str(CANONICAL).expect("deserialization must succeed");
                let expected: KeyValue = value();
                assert_eq!(actual, expected);
            }

            #[test]
            fn deserialize_alternative() {
                let actual: KeyValue =
                    serde_json::from_str(ALTERNATIVE).expect("deserialization must succeed");
                let expected: KeyValue = value();
                assert_eq!(actual, expected);
            }
        }
    }

    #[cfg(feature = "trace")]
    mod event {
        use super::*;

        mod simple {
            use super::*;

            fn value() -> Event {
                Event {
                    time_unix_nano: 1234567890,
                    name: String::from("my_event"),
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }
            }

            // language=json
            const CANONICAL: &str = r#"{
  "timeUnixNano": "1234567890",
  "name": "my_event",
  "attributes": [],
  "droppedAttributesCount": 0
}"#;

            // language=json
            const ALTERNATIVE: &str = r#"{
  "name": "my_event",
  "timeUnixNano": "1234567890"
}"#;

            #[test]
            fn serialize() {
                let input: Event = value();
                let actual =
                    serde_json::to_string_pretty(&input).expect("serialization must succeed");
                assert_eq!(actual, CANONICAL);
            }

            #[test]
            fn deserialize_canonical() {
                let actual: Event =
                    serde_json::from_str(CANONICAL).expect("deserialization must succeed");
                let expected: Event = value();
                assert_eq!(actual, expected);
            }

            #[test]
            fn deserialize_alternative() {
                let actual: Event =
                    serde_json::from_str(ALTERNATIVE).expect("deserialization must succeed");
                let expected: Event = value();
                assert_eq!(actual, expected);
            }
        }
    }

    #[cfg(feature = "metrics")]
    mod export_metrics_service_request {
        use super::*;

        // `ExportTraceServiceRequest` from the OpenTelemetry proto examples
        // see <https://github.com/open-telemetry/opentelemetry-proto/blob/v1.3.2/examples/metrics.json>
        mod example {
            use super::*;

            fn value() -> ExportMetricsServiceRequest {
                ExportMetricsServiceRequest {
                    resource_metrics: vec![ResourceMetrics {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: String::from("service.name"),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue(String::from("my.service"))),
                                }),
                                key_strindex: 0,
                            }],
                            dropped_attributes_count: 0,
                            entity_refs: vec![],
                        }),
                        scope_metrics: vec![ScopeMetrics {
                            scope: Some(InstrumentationScope {
                                name: String::from("my.library"),
                                version: String::from("1.0.0"),
                                attributes: vec![KeyValue {
                                    key: String::from("my.scope.attribute"),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue(String::from(
                                            "some scope attribute",
                                        ))),
                                    }),
                                    key_strindex: 0,
                                }],
                                dropped_attributes_count: 0,
                            }),
                            metrics: vec![
                                Metric {
                                    name: String::from("my.counter"),
                                    description: String::from("I am a Counter"),
                                    unit: String::from("1"),
                                    metadata: vec![],
                                    data: Some(Data::Sum(Sum {
                                        data_points: vec![NumberDataPoint {
                                            attributes: vec![KeyValue {
                                                key: String::from("my.counter.attr"),
                                                value: Some(AnyValue {
                                                    value: Some(Value::StringValue(String::from(
                                                        "some value",
                                                    ))),
                                                }),
                                                key_strindex: 0,
                                            }],
                                            start_time_unix_nano: 1544712660300000000,
                                            time_unix_nano: 1544712660300000000,
                                            exemplars: vec![],
                                            flags: 0,
                                            value: Some(MetricValue::AsDouble(5.0)),
                                        }],
                                        aggregation_temporality: 1,
                                        is_monotonic: true,
                                    })),
                                },
                                Metric {
                                    name: String::from("my.gauge"),
                                    description: String::from("I am a Gauge"),
                                    unit: String::from("1"),
                                    metadata: vec![],
                                    data: Some(Data::Gauge(Gauge {
                                        data_points: vec![NumberDataPoint {
                                            attributes: vec![KeyValue {
                                                key: String::from("my.gauge.attr"),
                                                value: Some(AnyValue {
                                                    value: Some(Value::StringValue(String::from(
                                                        "some value",
                                                    ))),
                                                }),
                                                key_strindex: 0,
                                            }],
                                            start_time_unix_nano: 0,
                                            time_unix_nano: 1544712660300000000,
                                            exemplars: vec![],
                                            flags: 0,
                                            value: Some(MetricValue::AsDouble(10.0)),
                                        }],
                                    })),
                                },
                                Metric {
                                    name: String::from("my.histogram"),
                                    description: String::from("I am a Histogram"),
                                    unit: String::from("1"),
                                    metadata: vec![],
                                    data: Some(Data::Histogram(Histogram {
                                        data_points: vec![HistogramDataPoint {
                                            attributes: vec![KeyValue {
                                                key: String::from("my.histogram.attr"),
                                                value: Some(AnyValue {
                                                    value: Some(Value::StringValue(String::from(
                                                        "some value",
                                                    ))),
                                                }),
                                                key_strindex: 0,
                                            }],
                                            start_time_unix_nano: 1544712660300000000,
                                            time_unix_nano: 1544712660300000000,
                                            count: 2,
                                            sum: Some(2.0),
                                            bucket_counts: vec![1, 1],
                                            explicit_bounds: vec![1.0],
                                            exemplars: vec![],
                                            flags: 0,
                                            min: Some(0.0),
                                            max: Some(2.0),
                                        }],
                                        aggregation_temporality: 1,
                                    })),
                                },
                            ],
                            schema_url: String::new(),
                        }],
                        schema_url: String::new(),
                    }],
                }
            }

            // language=json
            const CANONICAL: &str = r#"{
  "resourceMetrics": [
    {
      "resource": {
        "attributes": [
          {
            "key": "service.name",
            "value": {
              "stringValue": "my.service"
            }
          }
        ],
        "droppedAttributesCount": 0,
        "entityRefs": []
      },
      "scopeMetrics": [
        {
          "scope": {
            "name": "my.library",
            "version": "1.0.0",
            "attributes": [
              {
                "key": "my.scope.attribute",
                "value": {
                  "stringValue": "some scope attribute"
                }
              }
            ],
            "droppedAttributesCount": 0
          },
          "metrics": [
            {
              "name": "my.counter",
              "description": "I am a Counter",
              "unit": "1",
              "metadata": [],
              "sum": {
                "dataPoints": [
                  {
                    "attributes": [
                      {
                        "key": "my.counter.attr",
                        "value": {
                          "stringValue": "some value"
                        }
                      }
                    ],
                    "startTimeUnixNano": "1544712660300000000",
                    "timeUnixNano": "1544712660300000000",
                    "exemplars": [],
                    "flags": 0,
                    "asDouble": 5.0
                  }
                ],
                "aggregationTemporality": 1,
                "isMonotonic": true
              }
            },
            {
              "name": "my.gauge",
              "description": "I am a Gauge",
              "unit": "1",
              "metadata": [],
              "gauge": {
                "dataPoints": [
                  {
                    "attributes": [
                      {
                        "key": "my.gauge.attr",
                        "value": {
                          "stringValue": "some value"
                        }
                      }
                    ],
                    "startTimeUnixNano": "0",
                    "timeUnixNano": "1544712660300000000",
                    "exemplars": [],
                    "flags": 0,
                    "asDouble": 10.0
                  }
                ]
              }
            },
            {
              "name": "my.histogram",
              "description": "I am a Histogram",
              "unit": "1",
              "metadata": [],
              "histogram": {
                "dataPoints": [
                  {
                    "attributes": [
                      {
                        "key": "my.histogram.attr",
                        "value": {
                          "stringValue": "some value"
                        }
                      }
                    ],
                    "startTimeUnixNano": "1544712660300000000",
                    "timeUnixNano": "1544712660300000000",
                    "count": "2",
                    "sum": 2.0,
                    "bucketCounts": [
                      "1",
                      "1"
                    ],
                    "explicitBounds": [
                      1.0
                    ],
                    "exemplars": [],
                    "flags": 0,
                    "min": 0.0,
                    "max": 2.0
                  }
                ],
                "aggregationTemporality": 1
              }
            }
          ],
          "schemaUrl": ""
        }
      ],
      "schemaUrl": ""
    }
  ]
}"#;

            // copied from the example json file
            // language=json
            const ALTERNATIVE: &str = r#"{
  "resourceMetrics": [
    {
      "resource": {
        "attributes": [
          {
            "key": "service.name",
            "value": {
              "stringValue": "my.service"
            }
          }
        ]
      },
      "scopeMetrics": [
        {
          "scope": {
            "name": "my.library",
            "version": "1.0.0",
            "attributes": [
              {
                "key": "my.scope.attribute",
                "value": {
                  "stringValue": "some scope attribute"
                }
              }
            ]
          },
          "metrics": [
            {
              "name": "my.counter",
              "unit": "1",
              "description": "I am a Counter",
              "sum": {
                "aggregationTemporality": 1,
                "isMonotonic": true,
                "dataPoints": [
                  {
                    "asDouble": 5,
                    "startTimeUnixNano": "1544712660300000000",
                    "timeUnixNano": "1544712660300000000",
                    "attributes": [
                      {
                        "key": "my.counter.attr",
                        "value": {
                          "stringValue": "some value"
                        }
                      }
                    ]
                  }
                ]
              }
            },
            {
              "name": "my.gauge",
              "unit": "1",
              "description": "I am a Gauge",
              "gauge": {
                "dataPoints": [
                  {
                    "asDouble": 10,
                    "timeUnixNano": "1544712660300000000",
                    "attributes": [
                      {
                        "key": "my.gauge.attr",
                        "value": {
                          "stringValue": "some value"
                        }
                      }
                    ]
                  }
                ]
              }
            },
            {
              "name": "my.histogram",
              "unit": "1",
              "description": "I am a Histogram",
              "histogram": {
                "aggregationTemporality": 1,
                "dataPoints": [
                  {
                    "startTimeUnixNano": "1544712660300000000",
                    "timeUnixNano": "1544712660300000000",
                    "count": "2",
                    "sum": 2,
                    "bucketCounts": ["1","1"],
                    "explicitBounds": [1],
                    "min": 0,
                    "max": 2,
                    "attributes": [
                      {
                        "key": "my.histogram.attr",
                        "value": {
                          "stringValue": "some value"
                        }
                      }
                    ]
                  }
                ]
              }
            }
          ]
        }
      ]
    }
  ]
}
"#;

            #[test]
            fn serialize() {
                let input: ExportMetricsServiceRequest = value();
                let actual =
                    serde_json::to_string_pretty(&input).expect("serialization must succeed");
                assert_eq!(actual, CANONICAL);
            }

            #[test]
            fn deserialize_canonical() {
                let actual: ExportMetricsServiceRequest =
                    serde_json::from_str(CANONICAL).expect("deserialization must succeed");
                let expected: ExportMetricsServiceRequest = value();
                assert_eq!(actual, expected);
            }

            #[test]
            fn deserialize_alternative() {
                let actual: ExportMetricsServiceRequest =
                    serde_json::from_str(ALTERNATIVE).expect("deserialization must succeed");
                let expected: ExportMetricsServiceRequest = value();
                assert_eq!(actual, expected);
            }
        }
    }

    #[cfg(feature = "logs")]
    mod export_logs_service_request {
        use super::*;

        // `ExportTraceServiceRequest` from the OpenTelemetry proto examples
        // see <https://github.com/open-telemetry/opentelemetry-proto/blob/v1.3.2/examples/logs.json>
        mod example {
            use super::*;

            fn value() -> ExportLogsServiceRequest {
                ExportLogsServiceRequest {
                    resource_logs: vec![ResourceLogs {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: String::from("service.name"),
                                value: Some(AnyValue {
                                    value: Some(Value::StringValue(String::from("my.service"))),
                                }),
                                key_strindex: 0,
                            }],
                            dropped_attributes_count: 0,
                            entity_refs: vec![],
                        }),
                        scope_logs: vec![ScopeLogs {
                            scope: Some(InstrumentationScope {
                                name: String::from("my.library"),
                                version: String::from("1.0.0"),
                                attributes: vec![KeyValue {
                                    key: String::from("my.scope.attribute"),
                                    value: Some(AnyValue {
                                        value: Some(Value::StringValue(String::from(
                                            "some scope attribute",
                                        ))),
                                    }),
                                    key_strindex: 0,
                                }],
                                dropped_attributes_count: 0,
                            }),
                            log_records: vec![LogRecord {
                                time_unix_nano: 1544712660300000000,
                                observed_time_unix_nano: 1544712660300000000,
                                severity_number: 10,
                                severity_text: String::from("Information"),
                                body: Some(AnyValue {
                                    value: Some(Value::StringValue(String::from(
                                        "Example log record",
                                    ))),
                                }),
                                event_name: "test_log_event".to_string(),
                                attributes: vec![
                                    KeyValue {
                                        key: String::from("string.attribute"),
                                        value: Some(AnyValue {
                                            value: Some(Value::StringValue(String::from(
                                                "some string",
                                            ))),
                                        }),
                                        key_strindex: 0,
                                    },
                                    KeyValue {
                                        key: String::from("boolean.attribute"),
                                        value: Some(AnyValue {
                                            value: Some(Value::BoolValue(true)),
                                        }),
                                        key_strindex: 0,
                                    },
                                    KeyValue {
                                        key: String::from("int.attribute"),
                                        value: Some(AnyValue {
                                            value: Some(Value::IntValue(10)),
                                        }),
                                        key_strindex: 0,
                                    },
                                    KeyValue {
                                        key: String::from("double.attribute"),
                                        value: Some(AnyValue {
                                            value: Some(Value::DoubleValue(637.704)),
                                        }),
                                        key_strindex: 0,
                                    },
                                    KeyValue {
                                        key: String::from("array.attribute"),
                                        value: Some(AnyValue {
                                            value: Some(Value::ArrayValue(ArrayValue {
                                                values: vec![
                                                    AnyValue {
                                                        value: Some(Value::StringValue(
                                                            String::from("many"),
                                                        )),
                                                    },
                                                    AnyValue {
                                                        value: Some(Value::StringValue(
                                                            String::from("values"),
                                                        )),
                                                    },
                                                ],
                                            })),
                                        }),
                                        key_strindex: 0,
                                    },
                                    KeyValue {
                                        key: String::from("map.attribute"),
                                        value: Some(AnyValue {
                                            value: Some(Value::KvlistValue(KeyValueList {
                                                values: vec![KeyValue {
                                                    key: String::from("some.map.key"),
                                                    value: Some(AnyValue {
                                                        value: Some(Value::StringValue(
                                                            String::from("some value"),
                                                        )),
                                                    }),
                                                    key_strindex: 0,
                                                }],
                                            })),
                                        }),
                                        key_strindex: 0,
                                    },
                                ],
                                dropped_attributes_count: 0,
                                flags: 0,
                                trace_id: const_hex::decode("5b8efff798038103d269b633813fc60c")
                                    .unwrap(),
                                span_id: const_hex::decode("eee19b7ec3c1b174").unwrap(),
                            }],
                            schema_url: String::new(),
                        }],
                        schema_url: String::new(),
                    }],
                }
            }

            // language=json
            const CANONICAL: &str = r#"{
  "resourceLogs": [
    {
      "resource": {
        "attributes": [
          {
            "key": "service.name",
            "value": {
              "stringValue": "my.service"
            }
          }
        ],
        "droppedAttributesCount": 0,
        "entityRefs": []
      },
      "scopeLogs": [
        {
          "scope": {
            "name": "my.library",
            "version": "1.0.0",
            "attributes": [
              {
                "key": "my.scope.attribute",
                "value": {
                  "stringValue": "some scope attribute"
                }
              }
            ],
            "droppedAttributesCount": 0
          },
          "logRecords": [
            {
              "timeUnixNano": "1544712660300000000",
              "observedTimeUnixNano": "1544712660300000000",
              "severityNumber": 10,
              "severityText": "Information",
              "body": {
                "stringValue": "Example log record"
              },
              "attributes": [
                {
                  "key": "string.attribute",
                  "value": {
                    "stringValue": "some string"
                  }
                },
                {
                  "key": "boolean.attribute",
                  "value": {
                    "boolValue": true
                  }
                },
                {
                  "key": "int.attribute",
                  "value": {
                    "intValue": "10"
                  }
                },
                {
                  "key": "double.attribute",
                  "value": {
                    "doubleValue": 637.704
                  }
                },
                {
                  "key": "array.attribute",
                  "value": {
                    "arrayValue": {
                      "values": [
                        {
                          "stringValue": "many"
                        },
                        {
                          "stringValue": "values"
                        }
                      ]
                    }
                  }
                },
                {
                  "key": "map.attribute",
                  "value": {
                    "kvlistValue": {
                      "values": [
                        {
                          "key": "some.map.key",
                          "value": {
                            "stringValue": "some value"
                          }
                        }
                      ]
                    }
                  }
                }
              ],
              "droppedAttributesCount": 0,
              "flags": 0,
              "traceId": "5b8efff798038103d269b633813fc60c",
              "spanId": "eee19b7ec3c1b174",
              "eventName": "test_log_event"
            }
          ],
          "schemaUrl": ""
        }
      ],
      "schemaUrl": ""
    }
  ]
}"#;

            // copied from the example json file with fix for <https://github.com/open-telemetry/opentelemetry-proto/issues/579>
            // language=json
            const ALTERNATIVE: &str = r#"{
  "resourceLogs": [
    {
      "resource": {
        "attributes": [
          {
            "key": "service.name",
            "value": {
              "stringValue": "my.service"
            }
          }
        ]
      },
      "scopeLogs": [
        {
          "scope": {
            "name": "my.library",
            "version": "1.0.0",
            "attributes": [
              {
                "key": "my.scope.attribute",
                "value": {
                  "stringValue": "some scope attribute"
                }
              }
            ]
          },
          "logRecords": [
            {
              "timeUnixNano": "1544712660300000000",
              "observedTimeUnixNano": "1544712660300000000",
              "severityNumber": 10,
              "severityText": "Information",
              "traceId": "5B8EFFF798038103D269B633813FC60C",
              "spanId": "EEE19B7EC3C1B174",
              "body": {
                "stringValue": "Example log record"
              },
              "attributes": [
                {
                  "key": "string.attribute",
                  "value": {
                    "stringValue": "some string"
                  }
                },
                {
                  "key": "boolean.attribute",
                  "value": {
                    "boolValue": true
                  }
                },
                {
                  "key": "int.attribute",
                  "value": {
                    "intValue": "10"
                  }
                },
                {
                  "key": "double.attribute",
                  "value": {
                    "doubleValue": 637.704
                  }
                },
                {
                  "key": "array.attribute",
                  "value": {
                    "arrayValue": {
                      "values": [
                        {
                          "stringValue": "many"
                        },
                        {
                          "stringValue": "values"
                        }
                      ]
                    }
                  }
                },
                {
                  "key": "map.attribute",
                  "value": {
                    "kvlistValue": {
                      "values": [
                        {
                          "key": "some.map.key",
                          "value": {
                            "stringValue": "some value"
                          }
                        }
                      ]
                    }
                  }
                }
              ],
              "eventName": "test_log_event"
            }
          ]
        }
      ]
    }
  ]
}
"#;

            #[test]
            fn serialize() {
                let input: ExportLogsServiceRequest = value();
                let actual =
                    serde_json::to_string_pretty(&input).expect("serialization must succeed");
                assert_eq!(actual, CANONICAL);
            }

            #[test]
            fn deserialize_canonical() {
                let actual: ExportLogsServiceRequest =
                    serde_json::from_str(CANONICAL).expect("deserialization must succeed");
                let expected: ExportLogsServiceRequest = value();
                assert_eq!(actual, expected);
            }

            #[test]
            fn deserialize_alternative() {
                let actual: ExportLogsServiceRequest =
                    serde_json::from_str(ALTERNATIVE).expect("deserialization must succeed");
                let expected: ExportLogsServiceRequest = value();
                assert_eq!(actual, expected);
            }
        }
    }
    #[cfg(feature = "metrics")]
    mod metrics_with_nan {
        use super::*;
        use opentelemetry_proto::tonic::common::v1::any_value::Value;
        use opentelemetry_proto::tonic::metrics::v1::exponential_histogram_data_point::Buckets;
        use opentelemetry_proto::tonic::metrics::v1::summary_data_point::ValueAtQuantile;
        use opentelemetry_proto::tonic::metrics::v1::Exemplar;
        use opentelemetry_proto::tonic::metrics::v1::ExponentialHistogram;
        use opentelemetry_proto::tonic::metrics::v1::Summary;
        use opentelemetry_proto::tonic::metrics::v1::SummaryDataPoint;

        fn value_with_nan() -> ExportMetricsServiceRequest {
            ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics {
                    resource: Some(Resource {
                        attributes: vec![KeyValue {
                            key: String::from("service.name"),
                            value: None,
                            key_strindex: 0,
                        }],
                        dropped_attributes_count: 0,
                        entity_refs: vec![],
                    }),
                    scope_metrics: vec![ScopeMetrics {
                        scope: None,
                        metrics: vec![
                            Metric {
                                name: String::from("example_metric"),
                                description: String::from("A sample metric with NaN values"),
                                unit: String::from("1"),
                                metadata: vec![],
                                data: Some(
                                    opentelemetry_proto::tonic::metrics::v1::metric::Data::Summary(
                                        Summary {
                                            data_points: vec![SummaryDataPoint {
                                                attributes: vec![],
                                                start_time_unix_nano: 0,
                                                time_unix_nano: 0,
                                                count: 100,
                                                sum: f64::NAN,
                                                quantile_values: vec![
                                                    ValueAtQuantile {
                                                        quantile: 0.5,
                                                        value: f64::NAN,
                                                    },
                                                    ValueAtQuantile {
                                                        quantile: 0.9,
                                                        value: f64::NAN,
                                                    },
                                                ],
                                                flags: 0,
                                            }],
                                        },
                                    ),
                                ),
                            },
                            Metric {
                                name: String::from("my.histogram"),
                                description: String::from("I am an Histogram with NaN values"),
                                unit: String::from("1"),
                                metadata: vec![],
                                data: Some(Data::Histogram(Histogram {
                                    data_points: vec![HistogramDataPoint {
                                        attributes: vec![KeyValue {
                                            key: String::from("my.histogram.attr"),
                                            value: Some(AnyValue {
                                                value: Some(Value::StringValue(String::from(
                                                    "some value",
                                                ))),
                                            }),
                                            key_strindex: 0,
                                        }],
                                        start_time_unix_nano: 1544712660300000000,
                                        time_unix_nano: 1544712660300000000,
                                        count: 2,
                                        sum: Some(f64::NAN),
                                        bucket_counts: vec![1, 1],
                                        explicit_bounds: vec![f64::NAN],
                                        exemplars: vec![Exemplar {
                                            filtered_attributes: vec![KeyValue {
                                                key: String::from("my.histogram.attr"),
                                                value: Some(AnyValue {
                                                    value: Some(Value::DoubleValue(f64::NAN)),
                                                }),
                                                key_strindex: 0,
                                            }],
                                            time_unix_nano: 1544712660300000000,
                                            span_id: vec![],
                                            trace_id: vec![],
                                            value: Some(ExemplarValue::AsDouble(f64::NAN)),
                                        }],
                                        flags: 0,
                                        min: Some(f64::NAN),
                                        max: Some(f64::NAN),
                                    }],
                                    aggregation_temporality: 1,
                                })),
                            },
                            Metric {
                                name: String::from("my.exponential.histogram"),
                                description: String::from(
                                    "I am an exponential Histogram with NaN values",
                                ),
                                unit: String::from("1"),
                                metadata: vec![],
                                data: Some(Data::ExponentialHistogram(ExponentialHistogram {
                                    data_points: vec![ExponentialHistogramDataPoint {
                                        attributes: vec![KeyValue {
                                            key: String::from("my.exp.histogram.attr"),
                                            value: Some(AnyValue {
                                                value: Some(Value::StringValue(String::from(
                                                    "some value",
                                                ))),
                                            }),
                                            key_strindex: 0,
                                        }],
                                        start_time_unix_nano: 1544712660300000000,
                                        time_unix_nano: 1544712660300000000,
                                        count: 2,
                                        sum: Some(f64::NAN),
                                        scale: 1,
                                        zero_count: 0,
                                        exemplars: vec![Exemplar {
                                            filtered_attributes: vec![KeyValue {
                                                key: String::from("my.histogram.attr"),
                                                value: Some(AnyValue {
                                                    value: Some(Value::StringValue(String::from(
                                                        "some value",
                                                    ))),
                                                }),
                                                key_strindex: 0,
                                            }],
                                            time_unix_nano: 1544712660300000000,
                                            span_id: vec![],
                                            trace_id: vec![],
                                            value: Some(ExemplarValue::AsDouble(f64::NAN)),
                                        }],
                                        flags: 0,
                                        min: Some(f64::NAN),
                                        max: Some(f64::NAN),
                                        positive: Some(Buckets {
                                            offset: 0,
                                            bucket_counts: vec![],
                                        }),
                                        negative: Some(Buckets {
                                            offset: 0,
                                            bucket_counts: vec![],
                                        }),
                                        zero_threshold: f64::NAN,
                                    }],
                                    aggregation_temporality: 1,
                                })),
                            },
                            Metric {
                                name: String::from("my.counter"),
                                description: String::from("I am a Counter"),
                                unit: String::from("1"),
                                metadata: vec![],
                                data: Some(Data::Sum(Sum {
                                    data_points: vec![NumberDataPoint {
                                        attributes: vec![KeyValue {
                                            key: String::from("my.counter.attr"),
                                            value: Some(AnyValue {
                                                value: Some(Value::StringValue(String::from(
                                                    "some value",
                                                ))),
                                            }),
                                            key_strindex: 0,
                                        }],
                                        start_time_unix_nano: 1544712660300000000,
                                        time_unix_nano: 1544712660300000000,
                                        exemplars: vec![Exemplar {
                                            filtered_attributes: vec![KeyValue {
                                                key: String::from("my.histogram.attr"),
                                                value: Some(AnyValue {
                                                    value: Some(Value::StringValue(String::from(
                                                        "some value",
                                                    ))),
                                                }),
                                                key_strindex: 0,
                                            }],
                                            time_unix_nano: 1544712660300000000,
                                            span_id: vec![],
                                            trace_id: vec![],
                                            value: Some(ExemplarValue::AsDouble(f64::NAN)),
                                        }],
                                        flags: 0,
                                        value: Some(MetricValue::AsDouble(f64::NAN)),
                                    }],
                                    aggregation_temporality: 1,
                                    is_monotonic: true,
                                })),
                            },
                            Metric {
                                name: String::from("my.gauge"),
                                description: String::from("I am a Gauge"),
                                unit: String::from("1"),
                                metadata: vec![],
                                data: Some(Data::Gauge(Gauge {
                                    data_points: vec![NumberDataPoint {
                                        attributes: vec![KeyValue {
                                            key: String::from("my.gauge.attr"),
                                            value: Some(AnyValue {
                                                value: Some(Value::StringValue(String::from(
                                                    "some value",
                                                ))),
                                            }),
                                            key_strindex: 0,
                                        }],
                                        start_time_unix_nano: 0,
                                        time_unix_nano: 1544712660300000000,
                                        exemplars: vec![Exemplar {
                                            filtered_attributes: vec![KeyValue {
                                                key: String::from("my.histogram.attr"),
                                                value: Some(AnyValue {
                                                    value: Some(Value::StringValue(String::from(
                                                        "some value",
                                                    ))),
                                                }),
                                                key_strindex: 0,
                                            }],
                                            time_unix_nano: 1544712660300000000,
                                            span_id: vec![],
                                            trace_id: vec![],
                                            value: Some(ExemplarValue::AsDouble(f64::NAN)),
                                        }],
                                        flags: 0,
                                        value: Some(MetricValue::AsDouble(f64::NAN)),
                                    }],
                                })),
                            },
                        ],
                        schema_url: String::new(),
                    }],
                    schema_url: String::new(),
                }],
            }
        }

        // language=json
        const CANONICAL_WITH_NAN: &str = r#"{
          "resourceMetrics": [
            {
              "resource": {
                "attributes": [
                  {
                    "key": "service.name",
                    "value": null
                  }
                ],
                "droppedAttributesCount": 0,
                "entityRefs": []
              },
              "scopeMetrics": [
                {
                  "scope": null,
                  "metrics": [
                    {
                      "name": "example_metric",
                      "description": "A sample metric with NaN values",
                      "unit": "1",
                      "metadata": [],
                      "summary": {
                        "dataPoints": [
                          {
                            "attributes": [],
                            "startTimeUnixNano": "0",
                            "timeUnixNano": "0",
                            "count": "100",
                            "sum": "NaN",
                            "quantileValues": [
                              {
                                "quantile": 0.5,
                                "value": "NaN"
                              },
                              {
                                "quantile": 0.9,
                                "value": "NaN"
                              }
                            ],
                            "flags": 0
                          }
                        ]
                      }
                    },
                    {
                    "name": "my.histogram",
                    "description": "I am an Histogram with NaN values",
                    "unit": "1",
                    "metadata": [],
                    "histogram": {
                      "dataPoints": [
                        {
                          "attributes": [
                            {
                              "key": "my.histogram.attr",
                              "value": {
                                "stringValue": "some value"
                              }
                            }
                          ],
                          "startTimeUnixNano": "1544712660300000000",
                          "timeUnixNano": "1544712660300000000",
                          "count": "2",
                          "sum": "NaN",
                          "bucketCounts": [
                            "1",
                            "1"
                          ],
                          "explicitBounds": [
                            "NaN"
                          ],
                          "exemplars": [
                            {
                              "filteredAttributes": [
                                {
                                  "key": "my.histogram.attr",
                                  "value": {
                                    "doubleValue": "NaN"
                                  }
                                }
                              ],
                              "timeUnixNano": "1544712660300000000",
                              "traceId": "",
                              "spanId": "",
                              "value": {
                                "asDouble": "NaN"
                              }
                            }
                          ],
                          "flags": 0,
                          "min": "NaN",
                          "max": "NaN"
                        }
                      ],
                        "aggregationTemporality": 1
                      }
                    },
                    {
                      "name": "my.exponential.histogram",
                      "description": "I am an exponential Histogram with NaN values",
                      "unit": "1",
                      "metadata": [],
                      "exponentialHistogram": {
                        "dataPoints": [
                          {
                            "attributes": [
                              {
                                "key": "my.exp.histogram.attr",
                                "value": {
                                  "stringValue": "some value"
                                }
                              }
                            ],
                            "startTimeUnixNano": "1544712660300000000",
                            "timeUnixNano": "1544712660300000000",
                            "count": "2",
                            "sum": "NaN",
                            "scale": 1,
                            "zeroCount": "0",
                            "positive": {"offset":0,"bucketCounts":[]},
                            "negative": {"offset":0,"bucketCounts":[]},
                            "flags": 0,
                            "exemplars": [
                              {
                                "filteredAttributes": [
                                  {
                                    "key": "my.histogram.attr",
                                    "value": {
                                      "stringValue": "some value"
                                    }
                                  }
                                ],
                                "timeUnixNano": "1544712660300000000",
                                "traceId": "",
                                "spanId": "",
                                "value": {
                                  "asDouble": "NaN"
                                }
                              }
                            ],
                            "min": "NaN",
                            "max": "NaN",
                            "zeroThreshold": "NaN"
                          }
                        ],
                        "aggregationTemporality": 1
                      }
                    },
                    {
                      "name": "my.counter",
                      "description": "I am a Counter",
                      "unit": "1",
                      "metadata": [],
                      "sum": {
                        "dataPoints": [
                          {
                            "attributes": [
                              {
                                "key": "my.counter.attr",
                                "value": {
                                  "stringValue": "some value"
                                }
                              }
                            ],
                            "startTimeUnixNano": "1544712660300000000",
                            "timeUnixNano": "1544712660300000000",
                            "exemplars": [
                              {
                                "filteredAttributes": [
                                  {
                                    "key": "my.histogram.attr",
                                    "value": {
                                      "stringValue": "some value"
                                    }
                                  }
                                ],
                                "timeUnixNano": "1544712660300000000",
                                "traceId": "",
                                "spanId": "",
                                "value": {
                                  "asDouble": "NaN"
                                }
                              }
                            ],
                            "flags": 0,
                            "asDouble": "NaN"
                          }
                        ],
                        "aggregationTemporality": 1,
                        "isMonotonic": true
                      }
                    },
                    {
                      "name": "my.gauge",
                      "description": "I am a Gauge",
                      "unit": "1",
                      "metadata": [],
                      "gauge": {
                        "dataPoints": [
                          {
                            "attributes": [
                              {
                                "key": "my.gauge.attr",
                                "value": {
                                  "stringValue": "some value"
                                }
                              }
                            ],
                            "startTimeUnixNano": "0",
                            "timeUnixNano": "1544712660300000000",
                            "exemplars": [
                              {
                                "filteredAttributes": [
                                  {
                                    "key": "my.histogram.attr",
                                    "value": {
                                      "stringValue": "some value"
                                    }
                                  }
                                ],
                                "timeUnixNano": "1544712660300000000",
                                "traceId": "",
                                "spanId": "",
                                "value": {
                                  "asDouble": "NaN"
                                }
                              }
                            ],
                            "flags": 0,
                            "asDouble": "NaN"
                          }
                        ]
                      }
                    }
                  ],
                  "schemaUrl": ""
                }
              ],
              "schemaUrl": ""
            }
          ]
        }"#;

        #[test]
        fn serialize_with_nan() {
            let input: ExportMetricsServiceRequest = value_with_nan();

            // Serialize the structure to JSON
            let actual = serde_json::to_string_pretty(&input).expect("serialization must succeed");

            // Normalize both the actual and expected JSON for comparison
            let actual_value: serde_json::Value =
                serde_json::from_str(&actual).expect("valid JSON");
            let expected_value: serde_json::Value =
                serde_json::from_str(CANONICAL_WITH_NAN).expect("valid JSON");

            // Compare the normalized JSON values
            assert_eq!(actual_value, expected_value);
        }

        #[test]
        fn deserialize_with_nan() {
            let actual: ExportMetricsServiceRequest =
                serde_json::from_str(CANONICAL_WITH_NAN).expect("deserialization must succeed");

            // Ensure the deserialized structure matches the expected values
            assert_eq!(actual.resource_metrics.len(), 1);

            let resource_metric = &actual.resource_metrics[0];
            assert_eq!(
                resource_metric.resource.as_ref().unwrap().attributes.len(),
                1
            );
            assert_eq!(
                resource_metric.resource.as_ref().unwrap().attributes[0].key,
                "service.name"
            );
            assert!(resource_metric.resource.as_ref().unwrap().attributes[0]
                .value
                .is_none());

            assert_eq!(resource_metric.scope_metrics.len(), 1);

            let scope_metric = &resource_metric.scope_metrics[0];
            assert!(scope_metric.scope.is_none());
            assert_eq!(scope_metric.metrics.len(), 5);

            let metric = &scope_metric.metrics[0];
            assert_eq!(metric.name, "example_metric");
            assert_eq!(metric.description, "A sample metric with NaN values");
            assert_eq!(metric.unit, "1");

            if let Some(opentelemetry_proto::tonic::metrics::v1::metric::Data::Summary(summary)) =
                &metric.data
            {
                assert_eq!(summary.data_points.len(), 1);

                let data_point = &summary.data_points[0];
                assert_eq!(data_point.attributes.len(), 0);
                assert_eq!(data_point.start_time_unix_nano, 0);
                assert_eq!(data_point.time_unix_nano, 0);
                assert_eq!(data_point.count, 100);
                assert!(data_point.sum.is_nan());

                assert_eq!(data_point.quantile_values.len(), 2);

                // Verify that quantile values are NaN
                assert!(data_point.quantile_values[0].value.is_nan());
                assert!(data_point.quantile_values[0].quantile == 0.5);
                assert!(data_point.quantile_values[1].value.is_nan());
                assert!(data_point.quantile_values[1].quantile == 0.9);
            } else {
                panic!("Expected metric data to be of type Summary");
            }
            let histogram_metric = &actual.resource_metrics[0].scope_metrics[0].metrics[1];
            assert_eq!(histogram_metric.name, "my.histogram");
            assert_eq!(histogram_metric.unit, "1");
            if let Some(opentelemetry_proto::tonic::metrics::v1::metric::Data::Histogram(hist)) =
                &histogram_metric.data
            {
                assert_eq!(hist.data_points.len(), 1);
                let data_point = &hist.data_points[0];
                assert_eq!(data_point.attributes.len(), 1);
                assert_eq!(data_point.start_time_unix_nano, 1544712660300000000);
                assert_eq!(data_point.time_unix_nano, 1544712660300000000);
                assert_eq!(data_point.count, 2);

                // Checking special NaN quantile values
                assert!(data_point.sum.unwrap().is_nan());
                assert!(data_point.min.unwrap().is_nan());
                assert!(data_point.max.unwrap().is_nan());

                assert_eq!(data_point.exemplars.len(), 1);
                let exemplar = &data_point.exemplars[0];
                assert_eq!(exemplar.filtered_attributes.len(), 1);
                let attr = &exemplar.filtered_attributes[0];
                assert_eq!(attr.key, "my.histogram.attr");
                match &attr.value {
                    Some(opentelemetry_proto::tonic::common::v1::AnyValue {
                        value:
                            Some(opentelemetry_proto::tonic::common::v1::any_value::Value::DoubleValue(
                                val,
                            )),
                    }) => assert!(val.is_nan()),
                    _ => panic!("Expected double value NaN in filtered_attributes"),
                }
                match exemplar.value {
                    Some(opentelemetry_proto::tonic::metrics::v1::exemplar::Value::AsDouble(
                        val,
                    )) => assert!(val.is_nan()),
                    _ => panic!("Expected double value in exemplar"),
                }
            } else {
                panic!("Expected histogram data");
            }

            let exp_histogram_data_point_metric =
                &actual.resource_metrics[0].scope_metrics[0].metrics[2];
            assert_eq!(
                exp_histogram_data_point_metric.name,
                "my.exponential.histogram"
            );
            assert_eq!(
                exp_histogram_data_point_metric.description,
                "I am an exponential Histogram with NaN values"
            );
            assert_eq!(exp_histogram_data_point_metric.unit, "1");
            assert_eq!(exp_histogram_data_point_metric.metadata.len(), 0);

            if let Some(
                opentelemetry_proto::tonic::metrics::v1::metric::Data::ExponentialHistogram(
                    exp_hist,
                ),
            ) = &exp_histogram_data_point_metric.data
            {
                assert_eq!(exp_hist.data_points.len(), 1);
                let data_point = &exp_hist.data_points[0];
                assert_eq!(data_point.attributes.len(), 1);
                assert_eq!(data_point.start_time_unix_nano, 1544712660300000000);
                assert_eq!(data_point.time_unix_nano, 1544712660300000000);
                assert_eq!(data_point.count, 2);
                assert!(data_point.sum.unwrap().is_nan());
                assert_eq!(data_point.scale, 1);
                assert_eq!(data_point.zero_count, 0);
                assert_eq!(data_point.positive.as_ref().unwrap().offset, 0);
                assert_eq!(data_point.positive.as_ref().unwrap().bucket_counts.len(), 0);
                assert_eq!(data_point.negative.as_ref().unwrap().bucket_counts.len(), 0);
                assert_eq!(data_point.exemplars.len(), 1);
                let exemplar = &data_point.exemplars[0];
                match exemplar.value {
                    Some(ExemplarValue::AsDouble(val)) => assert!(val.is_nan()),
                    _ => panic!("Expected double value in exemplar"),
                }
                assert_eq!(data_point.flags, 0);
                assert!(data_point.min.unwrap().is_nan());
                assert!(data_point.max.unwrap().is_nan());
                assert!(data_point.zero_threshold.is_nan());
            } else {
                panic!("Expected ExponentialHistogram data")
            }
            let counter_metrics = &actual.resource_metrics[0].scope_metrics[0].metrics[3];
            assert_eq!(counter_metrics.name, "my.counter");
            assert_eq!(counter_metrics.description, "I am a Counter");
            assert_eq!(counter_metrics.unit, "1");
            assert_eq!(counter_metrics.metadata.len(), 0);
            if let Some(opentelemetry_proto::tonic::metrics::v1::metric::Data::Sum(
                summary_data_point,
            )) = &counter_metrics.data
            {
                let data_points = &summary_data_point.data_points[0];
                assert_eq!(data_points.attributes.len(), 1);
                assert_eq!(data_points.start_time_unix_nano, 1544712660300000000);
                assert_eq!(data_points.time_unix_nano, 1544712660300000000);
                assert_eq!(data_points.exemplars.len(), 1);
                let exemplar = &data_points.exemplars[0];
                match exemplar.value {
                    Some(ExemplarValue::AsDouble(val)) => assert!(val.is_nan()),
                    _ => panic!("Expected double value in exemplar"),
                }
                assert_eq!(data_points.flags, 0);
                match data_points.value {
                    Some(MetricValue::AsDouble(val)) => assert!(val.is_nan()),
                    Some(MetricValue::AsInt(_val)) => (),
                    None => panic!("Expected double value in counter"),
                }
            } else {
                panic!("Expected Sum data")
            }
            let gauge_metrics = &actual.resource_metrics[0].scope_metrics[0].metrics[4];
            assert_eq!(gauge_metrics.name, "my.gauge");
            assert_eq!(gauge_metrics.description, "I am a Gauge");
            assert_eq!(gauge_metrics.unit, "1");
            assert_eq!(gauge_metrics.metadata.len(), 0);
            if let Some(opentelemetry_proto::tonic::metrics::v1::metric::Data::Gauge(
                summary_data_point,
            )) = &gauge_metrics.data
            {
                let data_points = &summary_data_point.data_points[0];
                assert_eq!(data_points.attributes.len(), 1);
                assert_eq!(data_points.start_time_unix_nano, 0);
                assert_eq!(data_points.time_unix_nano, 1544712660300000000);
                assert_eq!(data_points.exemplars.len(), 1);
                let exemplar = &data_points.exemplars[0];
                match exemplar.value {
                    Some(ExemplarValue::AsDouble(val)) => assert!(val.is_nan()),
                    _ => panic!("Expected double value in exemplar"),
                }
                assert_eq!(data_points.flags, 0);
                match data_points.value {
                    Some(MetricValue::AsDouble(val)) => assert!(val.is_nan()),
                    Some(MetricValue::AsInt(_val)) => (),
                    None => panic!("Expected double value in counter"),
                }
            } else {
                panic!("Expected gauge data")
            }
        }
    }

    #[cfg(feature = "metrics")]
    mod bare_number_deserialization {
        use super::*;

        #[test]
        fn u64_bare_number() {
            // parsers must accept both bare and quoted numbers
            let json = r#"{
  "resourceMetrics": [
    {
      "scopeMetrics": [
        {
          "metrics": [
            {
              "name": "test",
              "gauge": {
                "dataPoints": [
                  {
                    "startTimeUnixNano": 1544712660000000000,
                    "timeUnixNano": "1544712661000000000",
                    "asInt": "42"
                  }
                ]
              }
            }
          ]
        }
      ]
    }
  ]
}"#;
            let result: ExportMetricsServiceRequest =
                serde_json::from_str(json).expect("bare u64 numbers must deserialize");
            let dp = &result.resource_metrics[0].scope_metrics[0].metrics[0];
            if let Some(Data::Gauge(gauge)) = &dp.data {
                assert_eq!(
                    gauge.data_points[0].start_time_unix_nano,
                    1544712660000000000
                );
                assert_eq!(gauge.data_points[0].time_unix_nano, 1544712661000000000);
            } else {
                panic!("expected gauge data");
            }
        }

        #[test]
        fn vec_u64_bare_numbers() {
            // bucket_counts should accept both quoted and bare numbers in arrays
            let json = r#"{
  "resourceMetrics": [
    {
      "scopeMetrics": [
        {
          "metrics": [
            {
              "name": "test_histogram",
              "histogram": {
                "dataPoints": [
                  {
                    "startTimeUnixNano": "0",
                    "timeUnixNano": "0",
                    "count": 10,
                    "sum": 100.0,
                    "bucketCounts": [1, "2", 3],
                    "explicitBounds": [10.0, 20.0]
                  }
                ],
                "aggregationTemporality": 2
              }
            }
          ]
        }
      ]
    }
  ]
}"#;
            let result: ExportMetricsServiceRequest =
                serde_json::from_str(json).expect("bare u64 vec numbers must deserialize");
            let dp = &result.resource_metrics[0].scope_metrics[0].metrics[0];
            if let Some(Data::Histogram(hist)) = &dp.data {
                assert_eq!(hist.data_points[0].bucket_counts, vec![1, 2, 3]);
                assert_eq!(hist.data_points[0].count, 10);
            } else {
                panic!("expected histogram data");
            }
        }

        #[test]
        fn f64_bare_number_in_summary_quantile() {
            // deserialize_f64_special should accept both quoted and bare numeric strings
            let json = r#"{
  "resourceMetrics": [
    {
      "scopeMetrics": [
        {
          "metrics": [
            {
              "name": "test_summary",
              "summary": {
                "dataPoints": [
                  {
                    "count": "1",
                    "sum": 100.0,
                    "quantileValues": [
                      {
                        "quantile": "0.5",
                        "value": 99.0
                      }
                    ]
                  }
                ]
              }
            }
          ]
        }
      ]
    }
  ]
}"#;
            let result: ExportMetricsServiceRequest =
                serde_json::from_str(json).expect("quoted f64 numbers must deserialize");
            let dp = &result.resource_metrics[0].scope_metrics[0].metrics[0];
            if let Some(opentelemetry_proto::tonic::metrics::v1::metric::Data::Summary(summary)) =
                &dp.data
            {
                let qv = &summary.data_points[0].quantile_values[0];
                // assert!((qv.quantile - 0.5).abs() < f64::EPSILON);
                // assert!((qv.value - 99.0).abs() < f64::EPSILON);
                assert_eq!(qv.quantile, 0.5);
                assert_eq!(qv.value, 99.0);
            } else {
                panic!("expected summary data");
            }
        }
    }

    #[cfg(feature = "metrics")]
    mod metrics_with_infinite {
        use super::*;
        use opentelemetry_proto::tonic::metrics::v1::Histogram;
        use opentelemetry_proto::tonic::metrics::v1::HistogramDataPoint;

        fn value_with_infinite() -> ExportMetricsServiceRequest {
            ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics {
                    resource: None,
                    scope_metrics: vec![ScopeMetrics {
                        scope: None,
                        metrics: vec![Metric {
                            name: String::from("infinite_metric"),
                            description: String::from("Metric with infinity values"),
                            unit: String::from("1"),
                            metadata: vec![],
                            data: Some(
                                opentelemetry_proto::tonic::metrics::v1::metric::Data::Histogram(
                                    Histogram {
                                        data_points: vec![HistogramDataPoint {
                                            attributes: vec![],
                                            start_time_unix_nano: 0,
                                            time_unix_nano: 0,
                                            count: 1,
                                            sum: Some(f64::INFINITY),
                                            bucket_counts: vec![1],
                                            explicit_bounds: vec![f64::NEG_INFINITY, f64::INFINITY],
                                            exemplars: vec![],
                                            flags: 0,
                                            min: Some(f64::NEG_INFINITY),
                                            max: Some(f64::INFINITY),
                                        }],
                                        aggregation_temporality: 1,
                                    },
                                ),
                            ),
                        }],
                        schema_url: String::new(),
                    }],
                    schema_url: String::new(),
                }],
            }
        }

        // language=json
        const CANONICAL_WITH_INFINITE: &str = r#"{
          "resourceMetrics": [
            {
              "resource": null,
              "scopeMetrics": [
                {
                  "scope": null,
                  "metrics": [
                    {
                      "name": "infinite_metric",
                      "description": "Metric with infinity values",
                      "unit": "1",
                      "metadata": [],
                      "histogram": {
                        "dataPoints": [
                          {
                            "attributes": [],
                            "startTimeUnixNano": "0",
                            "timeUnixNano": "0",
                            "count": "1",
                            "sum": "Infinity",
                            "bucketCounts": ["1"],
                            "explicitBounds": ["-Infinity","Infinity"],
                            "exemplars": [],
                            "flags": 0,
                            "min": "-Infinity",
                            "max": "Infinity"
                          }
                        ],
                        "aggregationTemporality": 1
                      }
                    }
                  ],
                  "schemaUrl": ""
                }
              ],
              "schemaUrl": ""
            }
          ]
        }"#;

        #[test]
        fn serialize_with_infinite_values() {
            let input = value_with_infinite();
            let actual = serde_json::to_string_pretty(&input).unwrap();
            let actual_value: serde_json::Value = serde_json::from_str(&actual).unwrap();
            let expected_value: serde_json::Value =
                serde_json::from_str(CANONICAL_WITH_INFINITE).unwrap();
            assert_eq!(actual_value, expected_value);
        }

        #[test]
        fn deserialize_with_infinite_values() {
            let actual: ExportMetricsServiceRequest = serde_json::from_str(CANONICAL_WITH_INFINITE)
                .expect("deserialization must succeed");

            let metric = &actual.resource_metrics[0].scope_metrics[0].metrics[0];
            if let Some(opentelemetry_proto::tonic::metrics::v1::metric::Data::Histogram(hist)) =
                &metric.data
            {
                let dp = &hist.data_points[0];
                assert_eq!(dp.sum, Some(f64::INFINITY));
                assert_eq!(dp.min, Some(f64::NEG_INFINITY));
                assert_eq!(dp.max, Some(f64::INFINITY));
            } else {
                panic!("Expected Histogram data");
            }
        }
    }
}
