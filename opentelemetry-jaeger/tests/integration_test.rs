#[cfg(feature = "integration_test")]
mod tests {
    use opentelemetry::sdk::trace::Tracer as SdkTracer;
    use opentelemetry::trace::{StatusCode, TraceContextExt, Tracer, TracerProvider};
    use opentelemetry::KeyValue;
    use opentelemetry_jaeger::testing::{
        jaeger_api_v2 as jaeger_api, jaeger_client::JaegerTestClient,
    };
    use std::collections::HashMap;

    // the sample application that will be traced.
    // Expect the following span relationship:
    //     ┌─────────┐
    //     │ Step-1  │────────────┐
    //     └───┬─────┘            │
    //         │                  │
    //     ┌───┴─────┐       ┌────┴────┐
    //     │ Step-2-1│       │ Step-2-2├───────────┐
    //     └─────────┘       └────┬────┘           │
    //                            │                │
    //                       ┌────┴─────┐      ┌───┴─────┐
    //                       │ Step-3-1 │      │ Step-3-2│
    //                       └──────────┘      └─────────┘
    async fn sample_application(tracer: &SdkTracer) {
        {
            tracer.in_span("step-1", |cx| {
                tracer.in_span("step-2-1", |_cx| {});
                tracer.in_span("step-2-2", |_cx| {
                    tracer.in_span("step-3-1", |cx| {
                        let span = cx.span();
                        span.set_status(StatusCode::Error, "")
                    });
                    tracer.in_span("step-3-2", |cx| {
                        cx.span()
                            .set_attribute(KeyValue::new("tag-3-2-1", "tag-value-3-2-1"))
                    })
                });
                cx.span()
                    .add_event("something happened", vec![KeyValue::new("key1", "value1")]);
            });
        }
    }

    // This tests requires a jaeger agent running on the localhost.
    // You can override the agent end point using OTEL_TEST_JAEGER_AGENT_ENDPOINT env var
    // You can override the query API endpoint using OTEL_TEST_JAEGER_ENDPOINT env var
    // Alternative you can run scripts/integration-test.sh from project root path.
    //
    #[test]
    #[ignore]
    fn integration_test() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("cannot start runtime");

        let agent_endpoint =
            option_env!("OTEL_TEST_JAEGER_AGENT_ENDPOINT").unwrap_or("localhost:6831");
        let query_api_endpoint =
            option_env!("OTEL_TEST_JAEGER_ENDPOINT").unwrap_or("http://localhost:16685");
        const SERVICE_NAME: &str = "opentelemetry_jaeger_integration_test";
        const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
        const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

        println!("{}, {}", agent_endpoint, query_api_endpoint);

        runtime.block_on(async {
            let tracer = opentelemetry_jaeger::new_agent_pipeline()
                .with_endpoint(agent_endpoint)
                .with_service_name(SERVICE_NAME)
                .install_batch(opentelemetry::runtime::Tokio)
                .expect("cannot create tracer using default configuration");

            sample_application(&tracer).await;

            tracer.provider().unwrap().force_flush();
        });

        runtime.block_on(async {
            // build client
            let mut client = JaegerTestClient::new(query_api_endpoint);
            assert!(
                client.contain_service(SERVICE_NAME).await,
                "jaeger cannot find service"
            );
            let spans = client.find_traces_from_services(SERVICE_NAME).await;
            assert_eq!(spans.len(), 5);

            for span in spans.iter() {
                assert_common_attributes(span, SERVICE_NAME, CRATE_NAME, CRATE_VERSION)
            }

            // convert to span name/operation name -> span map
            let span_map: HashMap<String, jaeger_api::Span> = spans
                .into_iter()
                .map(|spans| (spans.operation_name.clone(), spans))
                .collect();

            let step_1 = span_map.get("step-1").expect("cannot find step-1 span");
            assert_parent(step_1, None);
            assert_eq!(step_1.logs.len(), 1);

            let step_2_1 = span_map.get("step-2-1").expect("cannot find step-2-1 span");
            assert_parent(step_2_1, Some(step_1));

            let step_2_2 = span_map.get("step-2-2").expect("cannot find step-2-2 span");
            assert_parent(step_2_2, Some(step_1));

            let step_3_1 = span_map.get("step-3-1").expect("cannot find step-3-1 span");
            assert_parent(step_3_1, Some(step_2_2));
            assert_tags_contains(step_3_1, "otel.status_code", "ERROR");
            assert_tags_contains(step_3_1, "error", "true");
            assert_eq!(step_3_1.flags, 1);

            let step_3_2 = span_map
                .get("step-3-2")
                .expect("cannot find step 3-2 spans");
            assert_parent(step_3_2, Some(step_2_2));
            assert_tags_contains(step_3_2, "tag-3-2-1", "tag-value-3-2-1");
        });
    }

    fn assert_parent(span: &jaeger_api::Span, parent_span: Option<&jaeger_api::Span>) {
        let parent = span
            .references
            .iter()
            .filter(|span_ref| span_ref.ref_type == jaeger_api::SpanRefType::ChildOf as i32)
            .collect::<Vec<&jaeger_api::SpanRef>>();
        if let Some(parent_span) = parent_span {
            assert_eq!(parent.len(), 1);
            let parent = parent.get(0).unwrap();
            assert_eq!(parent.span_id, parent_span.span_id);
            assert_eq!(parent.trace_id, parent_span.trace_id);
        } else {
            assert!(parent.is_empty());
        }
    }

    fn assert_common_attributes<T>(
        span: &jaeger_api::Span,
        service_name: T,
        library_name: T,
        library_version: T,
    ) where
        T: Into<String>,
    {
        assert_eq!(
            span.process.as_ref().unwrap().service_name,
            service_name.into()
        );
        let mut library_metadata = span
            .tags
            .iter()
            .filter(|kvs| kvs.key == "otel.library.name" || kvs.key == "otel.library.version")
            .collect::<Vec<&jaeger_api::KeyValue>>();
        assert_eq!(library_metadata.len(), 2);
        if library_metadata.get(0).unwrap().key != "otel.library.name" {
            library_metadata.swap(0, 1)
        }
        assert_eq!(library_metadata.get(0).unwrap().v_str, library_name.into());
        assert_eq!(
            library_metadata.get(1).unwrap().v_str,
            library_version.into()
        );
    }

    fn assert_tags_contains<T>(span: &jaeger_api::Span, key: T, value: T)
    where
        T: Into<String>,
    {
        let key = key.into();
        let value = value.into();
        assert!(span
            .tags
            .iter()
            .map(|tag| {
                (tag.key.clone(), {
                    match tag.v_type {
                        0 => tag.v_str.to_string(),
                        1 => tag.v_bool.to_string(),
                        2 => tag.v_int64.to_string(),
                        3 => tag.v_float64.to_string(),
                        4 => std::str::from_utf8(&tag.v_binary).unwrap_or("").into(),
                        _ => "".to_string(),
                    }
                })
            })
            .any(|(tag_key, tag_value)| tag_key == key.clone() && tag_value == value.clone()));
    }
}
