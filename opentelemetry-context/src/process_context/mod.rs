//! Process context sharing via memory-mapped regions.
//!
//! Implements [OTEP-4719]: publishing SDK resource attributes to a named
//! memory mapping so external readers (e.g. the OpenTelemetry eBPF Profiler)
//! can discover them without direct integration.
//!
//! The process context is a singleton — only one may be active per process.
//!
//! This module is Linux-only. On other platforms, [`publish`] and [`unpublish`]
//! are no-ops.
//!
//! [OTEP-4719]: https://github.com/open-telemetry/oteps/pull/4719

#[cfg(all(target_os = "linux", target_has_atomic = "64"))]
mod linux;

use opentelemetry_sdk::Resource;

/// Convert an SDK [`Resource`] into a serialized proto `ProcessContext` payload.
fn encode_process_context(resource: &Resource) -> Vec<u8> {
    use opentelemetry_proto::tonic::processcontext::v1development::ProcessContext;
    use opentelemetry_proto::tonic::resource::v1::Resource as ProtoResource;
    use opentelemetry_proto::transform::common::tonic::Attributes;
    use prost::Message;

    let attributes: Attributes = resource
        .iter()
        .map(|(k, v)| opentelemetry::KeyValue::new(k.clone(), v.clone()))
        .collect::<Vec<_>>()
        .into();

    let ctx = ProcessContext {
        resource: Some(ProtoResource {
            attributes: attributes.0,
            dropped_attributes_count: 0,
            entity_refs: vec![],
        }),
        extra_attributes: vec![],
    };

    ctx.encode_to_vec()
}

/// Publish the given [`Resource`] as the process context.
///
/// On Linux, this creates (or updates) a named memory mapping containing
/// a protobuf-serialized representation of the resource attributes, making
/// it discoverable by external readers such as the OpenTelemetry eBPF Profiler.
///
/// On non-Linux platforms, this is a no-op.
///
/// # Example
///
/// ```no_run
/// use opentelemetry_sdk::Resource;
///
/// let resource = Resource::builder()
///     .with_service_name("my-service")
///     .build();
///
/// opentelemetry_context::process_context::publish(&resource);
/// ```
pub fn publish(resource: &Resource) {
    let payload = encode_process_context(resource);

    #[cfg(all(target_os = "linux", target_has_atomic = "64"))]
    {
        if let Err(e) = linux::publish_raw_payload(payload) {
            opentelemetry::otel_warn!(
                name: "process_context.publish.failed",
                message = format!("{e}")
            );
        }
    }

    #[cfg(not(all(target_os = "linux", target_has_atomic = "64")))]
    {
        let _ = payload;
    }
}

/// Unpublish the process context, unmapping the shared memory region.
///
/// On non-Linux platforms, this is a no-op.
pub fn unpublish() {
    #[cfg(all(target_os = "linux", target_has_atomic = "64"))]
    {
        if let Err(e) = linux::unpublish() {
            opentelemetry::otel_warn!(
                name: "process_context.unpublish.failed",
                message = format!("{e}")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::KeyValue;

    #[test]
    fn encode_process_context_contains_resource_attributes() {
        let resource = Resource::builder_empty()
            .with_attributes([
                KeyValue::new("service.name", "test-service"),
                KeyValue::new("host.name", "test-host"),
            ])
            .build();

        let payload = encode_process_context(&resource);
        assert!(!payload.is_empty());

        // Verify the payload can be decoded back to a ProcessContext
        use opentelemetry_proto::tonic::processcontext::v1development::ProcessContext;
        use prost::Message;
        let ctx = ProcessContext::decode(payload.as_slice()).expect("failed to decode payload");
        let proto_resource = ctx.resource.expect("resource should be present");
        assert_eq!(proto_resource.attributes.len(), 2);
        assert!(ctx.extra_attributes.is_empty());
    }

    #[cfg(all(target_os = "linux", target_has_atomic = "64"))]
    #[serial_test::serial]
    mod linux_tests {
        use super::*;

        #[test]
        fn publish_and_unpublish_via_public_api() {
            let resource = Resource::builder_empty()
                .with_attributes([KeyValue::new("service.name", "test-service")])
                .build();

            // Should not panic
            publish(&resource);
            unpublish();
        }
    }
}
