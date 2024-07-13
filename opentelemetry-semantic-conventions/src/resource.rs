// DO NOT EDIT, this is an auto-generated file
//
// If you want to update the file:
// - Edit the template at scripts/templates/semantic_attributes.rs.j2
// - Run the script at scripts/generate-consts-from-spec.sh

//! # Resource Semantic Conventions
//!
//! The [resource semantic conventions] define a set of standardized attributes
//! to be used in `Resource`s.
//!
//! [resource semantic conventions]: https://github.com/open-telemetry/semantic-conventions/tree/main/model/resource
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::KeyValue;
//! use opentelemetry_sdk::{trace::{config, TracerProvider}, Resource};
//! use opentelemetry_semantic_conventions as semconv;
//!
//! let _tracer = TracerProvider::builder()
//!     .with_config(config().with_resource(Resource::new(vec![
//!         KeyValue::new(semconv::resource::SERVICE_NAME, "my-service"),
//!         KeyValue::new(semconv::resource::SERVICE_NAMESPACE, "my-namespace"),
//!     ])))
//!     .build();
//! ```
pub use crate::attribute::ANDROID_OS_API_LEVEL;
pub use crate::attribute::AWS_ECS_CLUSTER_ARN;
pub use crate::attribute::AWS_ECS_CONTAINER_ARN;
pub use crate::attribute::AWS_ECS_LAUNCHTYPE;
pub use crate::attribute::AWS_ECS_TASK_ARN;
pub use crate::attribute::AWS_ECS_TASK_FAMILY;
pub use crate::attribute::AWS_ECS_TASK_ID;
pub use crate::attribute::AWS_ECS_TASK_REVISION;
pub use crate::attribute::AWS_EKS_CLUSTER_ARN;
pub use crate::attribute::AWS_LOG_GROUP_ARNS;
pub use crate::attribute::AWS_LOG_GROUP_NAMES;
pub use crate::attribute::AWS_LOG_STREAM_ARNS;
pub use crate::attribute::AWS_LOG_STREAM_NAMES;
pub use crate::attribute::BROWSER_BRANDS;
pub use crate::attribute::BROWSER_LANGUAGE;
pub use crate::attribute::BROWSER_MOBILE;
pub use crate::attribute::BROWSER_PLATFORM;
pub use crate::attribute::CLOUD_ACCOUNT_ID;
pub use crate::attribute::CLOUD_AVAILABILITY_ZONE;
pub use crate::attribute::CLOUD_PLATFORM;
pub use crate::attribute::CLOUD_PROVIDER;
pub use crate::attribute::CLOUD_REGION;
pub use crate::attribute::CLOUD_RESOURCE_ID;
pub use crate::attribute::CONTAINER_COMMAND;
pub use crate::attribute::CONTAINER_COMMAND_ARGS;
pub use crate::attribute::CONTAINER_COMMAND_LINE;
pub use crate::attribute::CONTAINER_ID;
pub use crate::attribute::CONTAINER_IMAGE_ID;
pub use crate::attribute::CONTAINER_IMAGE_NAME;
pub use crate::attribute::CONTAINER_IMAGE_REPO_DIGESTS;
pub use crate::attribute::CONTAINER_IMAGE_TAGS;
pub use crate::attribute::CONTAINER_NAME;
pub use crate::attribute::CONTAINER_RUNTIME;
pub use crate::attribute::DEPLOYMENT_ENVIRONMENT;
pub use crate::attribute::DEVICE_ID;
pub use crate::attribute::DEVICE_MANUFACTURER;
pub use crate::attribute::DEVICE_MODEL_IDENTIFIER;
pub use crate::attribute::DEVICE_MODEL_NAME;
pub use crate::attribute::FAAS_INSTANCE;
pub use crate::attribute::FAAS_MAX_MEMORY;
pub use crate::attribute::FAAS_NAME;
pub use crate::attribute::FAAS_VERSION;
pub use crate::attribute::GCP_CLOUD_RUN_JOB_EXECUTION;
pub use crate::attribute::GCP_CLOUD_RUN_JOB_TASK_INDEX;
pub use crate::attribute::GCP_GCE_INSTANCE_HOSTNAME;
pub use crate::attribute::GCP_GCE_INSTANCE_NAME;
pub use crate::attribute::HEROKU_APP_ID;
pub use crate::attribute::HEROKU_RELEASE_COMMIT;
pub use crate::attribute::HEROKU_RELEASE_CREATION_TIMESTAMP;
pub use crate::attribute::HOST_ARCH;
pub use crate::attribute::HOST_CPU_CACHE_L2_SIZE;
pub use crate::attribute::HOST_CPU_FAMILY;
pub use crate::attribute::HOST_CPU_MODEL_ID;
pub use crate::attribute::HOST_CPU_MODEL_NAME;
pub use crate::attribute::HOST_CPU_STEPPING;
pub use crate::attribute::HOST_CPU_VENDOR_ID;
pub use crate::attribute::HOST_ID;
pub use crate::attribute::HOST_IMAGE_ID;
pub use crate::attribute::HOST_IMAGE_NAME;
pub use crate::attribute::HOST_IMAGE_VERSION;
pub use crate::attribute::HOST_IP;
pub use crate::attribute::HOST_MAC;
pub use crate::attribute::HOST_NAME;
pub use crate::attribute::HOST_TYPE;
pub use crate::attribute::K8S_CLUSTER_NAME;
pub use crate::attribute::K8S_CLUSTER_UID;
pub use crate::attribute::K8S_CONTAINER_NAME;
pub use crate::attribute::K8S_CONTAINER_RESTART_COUNT;
pub use crate::attribute::K8S_CONTAINER_STATUS_LAST_TERMINATED_REASON;
pub use crate::attribute::K8S_CRONJOB_NAME;
pub use crate::attribute::K8S_CRONJOB_UID;
pub use crate::attribute::K8S_DAEMONSET_NAME;
pub use crate::attribute::K8S_DAEMONSET_UID;
pub use crate::attribute::K8S_DEPLOYMENT_NAME;
pub use crate::attribute::K8S_DEPLOYMENT_UID;
pub use crate::attribute::K8S_JOB_NAME;
pub use crate::attribute::K8S_JOB_UID;
pub use crate::attribute::K8S_NAMESPACE_NAME;
pub use crate::attribute::K8S_NODE_NAME;
pub use crate::attribute::K8S_NODE_UID;
pub use crate::attribute::K8S_POD_NAME;
pub use crate::attribute::K8S_POD_UID;
pub use crate::attribute::K8S_REPLICASET_NAME;
pub use crate::attribute::K8S_REPLICASET_UID;
pub use crate::attribute::K8S_STATEFULSET_NAME;
pub use crate::attribute::K8S_STATEFULSET_UID;
pub use crate::attribute::OCI_MANIFEST_DIGEST;
pub use crate::attribute::OS_BUILD_ID;
pub use crate::attribute::OS_DESCRIPTION;
pub use crate::attribute::OS_NAME;
pub use crate::attribute::OS_TYPE;
pub use crate::attribute::OS_VERSION;
pub use crate::attribute::OTEL_SCOPE_NAME;
pub use crate::attribute::OTEL_SCOPE_VERSION;
pub use crate::attribute::PROCESS_COMMAND;
pub use crate::attribute::PROCESS_COMMAND_ARGS;
pub use crate::attribute::PROCESS_COMMAND_LINE;
pub use crate::attribute::PROCESS_EXECUTABLE_NAME;
pub use crate::attribute::PROCESS_EXECUTABLE_PATH;
pub use crate::attribute::PROCESS_OWNER;
pub use crate::attribute::PROCESS_PARENT_PID;
pub use crate::attribute::PROCESS_PID;
pub use crate::attribute::PROCESS_RUNTIME_DESCRIPTION;
pub use crate::attribute::PROCESS_RUNTIME_NAME;
pub use crate::attribute::PROCESS_RUNTIME_VERSION;
pub use crate::attribute::SERVICE_INSTANCE_ID;
pub use crate::attribute::SERVICE_NAME;
pub use crate::attribute::SERVICE_NAMESPACE;
pub use crate::attribute::SERVICE_VERSION;
pub use crate::attribute::TELEMETRY_DISTRO_NAME;
pub use crate::attribute::TELEMETRY_DISTRO_VERSION;
pub use crate::attribute::TELEMETRY_SDK_LANGUAGE;
pub use crate::attribute::TELEMETRY_SDK_NAME;
pub use crate::attribute::TELEMETRY_SDK_VERSION;
pub use crate::attribute::USER_AGENT_ORIGINAL;
pub use crate::attribute::WEBENGINE_DESCRIPTION;
pub use crate::attribute::WEBENGINE_NAME;
pub use crate::attribute::WEBENGINE_VERSION;
