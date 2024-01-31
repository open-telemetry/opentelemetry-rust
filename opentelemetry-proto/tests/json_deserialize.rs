#[cfg(all(feature = "with-serde", feature = "gen-tonic-messages"))]
mod json_deserialize {
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::common::v1::any_value::Value;
    use opentelemetry_proto::tonic::common::v1::KeyValue;
    use opentelemetry_proto::tonic::trace::v1::span::Event;

    // copied from example json file
    // see https://github.com/open-telemetry/opentelemetry-proto/blob/v1.0.0/examples/trace.json
    const TRACES_JSON: &str = r#"
    {
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
              "startTimeUnixNano": 1544712660000000000,
              "endTimeUnixNano": 1544712661000000000,
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

    const KEY_VALUES_JSON: &str = r#"
    {
            "key": "service.name",
            "value": {
              "stringValue": "my.service"
            }
          }
    "#;

    const EVENT_JSON: &str = r#"
    {
        "name": "my_event",
        "time_unix_nano": 1234567890
    }
    "#;

    #[test]
    fn test_deserialize_traces() {
        let request: ExportTraceServiceRequest = serde_json::from_str(TRACES_JSON).unwrap();
        assert_eq!(
            request.resource_spans[0].scope_spans[0].spans[0].trace_id,
            hex::decode("5B8EFFF798038103D269B633813FC60C").unwrap()
        )
    }

    #[test]
    fn test_deserialize_values() {
        // strings
        {
            let value: Value = serde_json::from_str(
                r#"
            {
              "stringValue": "my.service"
            }
        "#,
            )
            .unwrap();
            assert_eq!(value, Value::StringValue("my.service".to_string()));
        }
        // bools
        {
            let value: Value = serde_json::from_str(
                r#"
            {
              "boolValue": true
            }
        "#,
            )
            .unwrap();
            assert_eq!(value, Value::BoolValue(true));
        }
        // ints
        {
            let value: Value = serde_json::from_str(
                r#"
            {
              "intValue": 123
            }"#,
            )
            .unwrap();
            assert_eq!(value, Value::IntValue(123));
        }
        // doubles
        {
            let value: Value = serde_json::from_str(
                r#"
            {
              "doubleValue": 123.456
            }"#,
            )
            .unwrap();
            assert_eq!(value, Value::DoubleValue(123.456));
        }
        // todo(zhongyang): add tests for arrays and objects(need an example from other language)
    }

    #[test]
    fn test_deserialize_key_values() {
        let keyvalue: KeyValue = serde_json::from_str(KEY_VALUES_JSON).unwrap();

        assert_eq!(keyvalue.key, "service.name".to_string());
        assert_eq!(
            keyvalue.value.unwrap().value.unwrap(),
            Value::StringValue("my.service".to_string())
        );
    }

    #[test]
    fn test_event() {
        let event_json: Event = serde_json::from_str(EVENT_JSON).unwrap();
        assert_eq!(event_json.name, "my_event".to_string());
        assert_eq!(event_json.attributes.len(), 0);
    }
}
