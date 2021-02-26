//! # OpenTelemetry Datadog Exporter
//!
//! An OpenTelemetry exporter implementation
//!
//! See the [Datadog Docs](https://docs.datadoghq.com/agent/) for information on how to run the datadog-agent
//!
//! ## Quirks
//!
//! There are currently some incompatibilities between Datadog and OpenTelemetry, and this manifests
//! as minor quirks to this exporter.
//!
//! Firstly Datadog uses operation_name to describe what OpenTracing would call a component.
//! Or to put it another way, in OpenTracing the operation / span name's are relatively
//! granular and might be used to identify a specific endpoint. In datadog, however, they
//! are less granular - it is expected in Datadog that a service will have single
//! primary span name that is the root of all traces within that service, with an additional piece of
//! metadata called resource_name providing granularity - https://docs.datadoghq.com/tracing/guide/configuring-primary-operation/
//!
//! The Datadog Golang API takes the approach of using a `resource.name` OpenTelemetry attribute to set the
//! resource_name - https://github.com/DataDog/dd-trace-go/blob/ecb0b805ef25b00888a2fb62d465a5aa95e7301e/ddtrace/opentracer/tracer.go#L10
//!
//! Unfortunately, this breaks compatibility with other OpenTelemetry exporters which expect
//! a more granular operation name - as per the OpenTracing specification.
//!
//! This exporter therefore takes a different approach of naming the span with the name of the
//! tracing provider, and using the span name to set the resource_name. This should in most cases
//! lead to the behaviour that users expect.
//!
//! Datadog additionally has a span_type string that alters the rendering of the spans in the web UI.
//! This can be set as the `span.type` OpenTelemetry span attribute.
//!
//! For standard values see here - https://github.com/DataDog/dd-trace-go/blob/ecb0b805ef25b00888a2fb62d465a5aa95e7301e/ddtrace/ext/app_types.go#L31
//!
//! ## Performance
//!
//! For optimal performance, a batch exporter is recommended as the simple
//! exporter will export each span synchronously on drop. You can enable the
//! [`rt-tokio`], [`rt-tokio-current-thread`] or [`rt-async-std`] features to have a batch exporter configured for
//! you automatically for either executor when you install the pipeline.
//!
//! ```toml
//! [dependencies]
//! opentelemetry = { version = "*", features = ["rt-tokio"] }
//! opentelemetry-datadog = "*"
//! ```
//!
//! [`rt-tokio`]: https://tokio.rs
//! [`async-std`]: https://async.rs
//!

//! ## Bring your own http client
//!
//! Users can choose appropriate http clients to align with their runtime.
//!
//! Based on the feature enabled. The default http client will be different. If user doesn't specific
//! features or enabled `reqwest-blocking-client` feature. The blocking reqwest http client will be used as
//! default client. If `reqwest-client` feature is enabled. The async reqwest http client will be used. If
//! `surf-client` feature is enabled. The surf http client will be used.
//!
//! Note that async http clients may need specific runtime otherwise it will panic. User should make
//! sure the http client is running in appropriate runime.
//!
//! Users can always use their own http clients by implementing `HttpClient` trait.
//!
//! ## Kitchen Sink Full Configuration
//!
//! Example showing how to override all configuration options. See the
//! [`DatadogPipelineBuilder`] docs for details of each option.
//!
//! [`DatadogPipelineBuilder`]: struct.DatadogPipelineBuilder.html
//!
//! ```no_run
//! use opentelemetry::{KeyValue, trace::Tracer};
//! use opentelemetry::sdk::{trace::{self, IdGenerator, Sampler}, Resource};
//! use opentelemetry::sdk::export::trace::ExportResult;
//! use opentelemetry_datadog::{new_pipeline, ApiVersion, Error};
//! use opentelemetry::global::shutdown_tracer_provider;
//! use opentelemetry_http::HttpClient;
//! use async_trait::async_trait;
//!
//! // `reqwest` and `surf` are supported through features, if you prefer an
//! // alternate http client you can add support by implementing `HttpClient` as
//! // shown here.
//! #[derive(Debug)]
//! struct IsahcClient(isahc::HttpClient);
//!
//! #[async_trait]
//! impl HttpClient for IsahcClient {
//!   async fn send(&self, request: http::Request<Vec<u8>>) -> ExportResult {
//!     let result = self.0.send_async(request).await.map_err(|err| Error::Other(err.to_string()))?;
//!
//!     if result.status().is_success() {
//!       Ok(())
//!     } else {
//!       Err(Error::Other(result.status().to_string()).into())
//!     }
//!   }
//! }
//!
//! fn main() -> Result<(), opentelemetry::trace::TraceError> {
//!     let tracer = new_pipeline()
//!         .with_service_name("my_app")
//!         .with_version(ApiVersion::Version05)
//!         .with_agent_endpoint("http://localhost:8126")
//!         .with_trace_config(
//!             trace::config()
//!                 .with_default_sampler(Sampler::AlwaysOn)
//!                 .with_id_generator(IdGenerator::default())
//!         )
//!         .install()?;
//!
//!     tracer.in_span("doing_work", |cx| {
//!         // Traced app logic here...
//!     });
//!
//!     shutdown_tracer_provider(); // sending remaining spans before exit
//!
//!     Ok(())
//! }
//! ```

mod exporter;

mod propagator {
    use opentelemetry::{
        propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
        trace::{
            SpanContext, SpanId, TraceContextExt, TraceId, TraceState, TRACE_FLAG_DEFERRED,
            TRACE_FLAG_NOT_SAMPLED, TRACE_FLAG_SAMPLED,
        },
        Context,
    };

    const DATADOG_TRACE_ID_HEADER: &str = "x-datadog-trace-id";
    const DATADOG_PARENT_ID_HEADER: &str = "x-datadog-parent-id";
    const DATADOG_SAMPLING_PRIORITY_HEADER: &str = "x-datadog-sampling-priority";

    lazy_static::lazy_static! {
        static ref DATADOG_HEADER_FIELDS: [String; 3] = [
            DATADOG_TRACE_ID_HEADER.to_string(),
            DATADOG_PARENT_ID_HEADER.to_string(),
            DATADOG_SAMPLING_PRIORITY_HEADER.to_string(),
        ];
    }

    enum SamplingPriority {
        UserReject = -1,
        AutoReject = 0,
        AutoKeep = 1,
        UserKeep = 2,
    }

    #[derive(Debug)]
    enum ExtractError {
        TraceId,
        SpanId,
        SamplingPriority,
    }

    /// Extracts and injects `SpanContext`s into `Extractor`s or `Injector`s using Datadog's header format.
    ///
    /// The Datadog header format does not have an explicit spec, but can be divined from the client libraries,
    /// such as [dd-trace-go]
    ///
    /// ## Example
    ///
    /// ```
    /// use opentelemetry::global;
    /// use opentelemetry_datadog::DatadogPropagator;
    ///
    /// global::set_text_map_propagator(DatadogPropagator::default());
    /// ```
    ///
    /// [dd-trace-go]: https://github.com/DataDog/dd-trace-go/blob/v1.28.0/ddtrace/tracer/textmap.go#L293
    #[derive(Clone, Debug, Default)]
    pub struct DatadogPropagator {
        _private: (),
    }

    impl DatadogPropagator {
        /// Creates a new `DatadogPropagator`.
        pub fn new() -> Self {
            DatadogPropagator::default()
        }

        fn extract_trace_id(&self, trace_id: &str) -> Result<TraceId, ExtractError> {
            u64::from_str_radix(trace_id, 10)
                .map(|id| TraceId::from_u128(id as u128))
                .map_err(|_| ExtractError::TraceId)
        }

        fn extract_span_id(&self, span_id: &str) -> Result<SpanId, ExtractError> {
            u64::from_str_radix(span_id, 10)
                .map(SpanId::from_u64)
                .map_err(|_| ExtractError::SpanId)
        }

        fn extract_sampling_priority(
            &self,
            sampling_priority: &str,
        ) -> Result<SamplingPriority, ExtractError> {
            let i = i32::from_str_radix(sampling_priority, 10)
                .map_err(|_| ExtractError::SamplingPriority)?;

            match i {
                -1 => Ok(SamplingPriority::UserReject),
                0 => Ok(SamplingPriority::AutoReject),
                1 => Ok(SamplingPriority::AutoKeep),
                2 => Ok(SamplingPriority::UserKeep),
                _ => Err(ExtractError::SamplingPriority),
            }
        }

        fn extract_span_context(
            &self,
            extractor: &dyn Extractor,
        ) -> Result<SpanContext, ExtractError> {
            let trace_id =
                self.extract_trace_id(extractor.get(DATADOG_TRACE_ID_HEADER).unwrap_or(""))?;
            // If we have a trace_id but can't get the parent span, we default it to invalid instead of completely erroring
            // out so that the rest of the spans aren't completely lost
            let span_id = self
                .extract_span_id(extractor.get(DATADOG_PARENT_ID_HEADER).unwrap_or(""))
                .unwrap_or_else(|_| SpanId::invalid());
            let sampling_priority = self.extract_sampling_priority(
                extractor
                    .get(DATADOG_SAMPLING_PRIORITY_HEADER)
                    .unwrap_or(""),
            );
            let sampled = match sampling_priority {
                Ok(SamplingPriority::UserReject) | Ok(SamplingPriority::AutoReject) => {
                    TRACE_FLAG_NOT_SAMPLED
                }
                Ok(SamplingPriority::UserKeep) | Ok(SamplingPriority::AutoKeep) => {
                    TRACE_FLAG_SAMPLED
                }
                // Treat the sampling as DEFERRED instead of erroring on extracting the span context
                Err(_) => TRACE_FLAG_DEFERRED,
            };

            let trace_state = TraceState::default();

            Ok(SpanContext::new(
                trace_id,
                span_id,
                sampled,
                true,
                trace_state,
            ))
        }
    }

    impl TextMapPropagator for DatadogPropagator {
        fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
            let span_context = cx.span().span_context();
            if span_context.is_valid() {
                injector.set(
                    DATADOG_TRACE_ID_HEADER,
                    (span_context.trace_id().to_u128() as u64).to_string(),
                );
                injector.set(
                    DATADOG_PARENT_ID_HEADER,
                    span_context.span_id().to_u64().to_string(),
                );

                if !span_context.is_deferred() {
                    let sampling_priority = if span_context.is_sampled() {
                        SamplingPriority::AutoKeep
                    } else {
                        SamplingPriority::AutoReject
                    };

                    injector.set(
                        DATADOG_SAMPLING_PRIORITY_HEADER,
                        (sampling_priority as i32).to_string(),
                    );
                }
            }
        }

        fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
            let extracted = self
                .extract_span_context(extractor)
                .unwrap_or_else(|_| SpanContext::empty_context());

            cx.with_remote_span_context(extracted)
        }

        fn fields(&self) -> FieldIter<'_> {
            FieldIter::new(DATADOG_HEADER_FIELDS.as_ref())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use opentelemetry::testing::trace::TestSpan;
        use opentelemetry::trace::TraceState;
        use std::collections::HashMap;

        #[rustfmt::skip]
        fn extract_test_data() -> Vec<(Vec<(&'static str, &'static str)>, SpanContext)> {
            vec![
                (vec![], SpanContext::empty_context()),
                (vec![(DATADOG_SAMPLING_PRIORITY_HEADER, "0")], SpanContext::empty_context()),
                (vec![(DATADOG_TRACE_ID_HEADER, "garbage")], SpanContext::empty_context()),
                (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "garbage")], SpanContext::new(TraceId::from_u128(1234), SpanId::invalid(), TRACE_FLAG_DEFERRED, true, TraceState::default())),
                (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_DEFERRED, true, TraceState::default())),
                (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12"), (DATADOG_SAMPLING_PRIORITY_HEADER, "0")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_NOT_SAMPLED, true, TraceState::default())),
                (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12"), (DATADOG_SAMPLING_PRIORITY_HEADER, "1")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_SAMPLED, true, TraceState::default())),
            ]
        }

        #[rustfmt::skip]
        fn inject_test_data() -> Vec<(Vec<(&'static str, &'static str)>, SpanContext)> {
            vec![
                (vec![], SpanContext::empty_context()),
                (vec![], SpanContext::new(TraceId::from_hex("garbage"), SpanId::invalid(), TRACE_FLAG_DEFERRED, true, TraceState::default())),
                (vec![], SpanContext::new(TraceId::from_hex("1234"), SpanId::invalid(), TRACE_FLAG_DEFERRED, true, TraceState::default())),
                (vec![], SpanContext::new(TraceId::from_hex("1234"), SpanId::invalid(), TRACE_FLAG_SAMPLED, true, TraceState::default())),
                (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_DEFERRED, true, TraceState::default())),
                (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12"), (DATADOG_SAMPLING_PRIORITY_HEADER, "0")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_NOT_SAMPLED, true, TraceState::default())),
                (vec![(DATADOG_TRACE_ID_HEADER, "1234"), (DATADOG_PARENT_ID_HEADER, "12"), (DATADOG_SAMPLING_PRIORITY_HEADER, "1")], SpanContext::new(TraceId::from_u128(1234), SpanId::from_u64(12), TRACE_FLAG_SAMPLED, true, TraceState::default())),
            ]
        }

        #[test]
        fn test_extract() {
            for (header_list, expected) in extract_test_data() {
                let map: HashMap<String, String> = header_list
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();

                let propagator = DatadogPropagator::default();
                let context = propagator.extract(&map);
                assert_eq!(context.remote_span_context(), Some(&expected));
            }
        }

        #[test]
        fn test_extract_empty() {
            let map: HashMap<String, String> = HashMap::new();
            let propagator = DatadogPropagator::default();
            let context = propagator.extract(&map);
            assert_eq!(
                context.remote_span_context(),
                Some(&SpanContext::empty_context())
            )
        }

        #[test]
        fn test_inject() {
            let propagator = DatadogPropagator::default();
            for (header_values, span_context) in inject_test_data() {
                let mut injector: HashMap<String, String> = HashMap::new();
                propagator.inject_context(
                    &Context::current_with_span(TestSpan(span_context)),
                    &mut injector,
                );

                if header_values.is_empty() {
                    assert!(injector.is_empty());
                } else {
                    for (k, v) in header_values.into_iter() {
                        let injected_value: Option<&String> = injector.get(k);
                        assert_eq!(injected_value, Some(&v.to_string()));
                        injector.remove(k);
                    }
                    assert!(injector.is_empty());
                }
            }
        }
    }
}

pub use exporter::{new_pipeline, ApiVersion, DatadogExporter, DatadogPipelineBuilder, Error};
pub use propagator::DatadogPropagator;
