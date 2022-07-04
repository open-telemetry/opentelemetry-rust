#[cfg(feature = "grpc-tonic")]
use opentelemetry_proto::tonic::{common::v1::KeyValue, resource::v1::Resource};
use std::cmp::Ordering;

#[derive(PartialEq)]
pub(crate) struct ResourceWrapper(opentelemetry::sdk::Resource);

impl From<opentelemetry::sdk::Resource> for ResourceWrapper {
    fn from(r: opentelemetry::sdk::Resource) -> Self {
        ResourceWrapper(r)
    }
}

impl Eq for ResourceWrapper {}

impl Ord for ResourceWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.len().cmp(&other.0.len())
    }
}

impl PartialOrd for ResourceWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.len().cmp(&other.0.len()))
    }
}

impl ResourceWrapper {
    #[cfg(all(feature = "grpc-tonic", feature = "metrics"))]
    // it's currently only used by metrics. Trace set this in opentelemtry-proto
    pub(crate) fn schema_url(&self) -> Option<&str> {
        self.0.schema_url()
    }
}

#[cfg(feature = "grpc-tonic")]
impl From<ResourceWrapper> for Resource {
    fn from(resource: ResourceWrapper) -> Self {
        Resource {
            attributes: resource
                .0
                .into_iter()
                .map(|(key, value)| KeyValue {
                    key: key.as_str().to_string(),
                    value: Some(value.into()),
                })
                .collect::<Vec<KeyValue>>(),
            dropped_attributes_count: 0,
        }
    }
}
