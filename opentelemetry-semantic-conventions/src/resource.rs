//! # Resource Semantic Conventions
//!
//! The [resource semantic conventions] define a set of standardized attributes
//! to be used in `Resource`s.
//!
//! [resource semantic conventions]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions
//!
//! ## Usage
//!
//! ```rust,no_run
//! use opentelemetry::sdk;
//! use opentelemetry_semantic_conventions as semcov;
//! use std::sync::Arc;
//!
//! let _tracer = opentelemetry::exporter::trace::stdout::new_pipeline()
//!     .with_trace_config(sdk::Config {
//!         resource: Arc::new(sdk::Resource::new(vec![
//!             semcov::resource::SERVICE_NAME.string("my-service"),
//!             semcov::resource::SERVICE_NAMESPACE.string("my-namespace"),
//!         ])),
//!         ..sdk::Config::default()
//!     })
//!     .install();
//! ```

use opentelemetry::api::Key;

/// Logical name of the service.
///
/// MUST be the same for all instances of horizontally scaled services.
pub const SERVICE_NAME: Key = Key::from_static_str("service.name");

/// A namespace for `service.name`.
///
/// A string value having a meaning that helps to distinguish a group of
/// services, for example the team name that owns a group of services.
/// `service.name` is expected to be unique within the same namespace. The field
/// is optional.
///
/// If `service.namespace` is not specified in the Resource then `service.name`
/// is expected to be unique for all services that have no explicit namespace
/// defined (so the empty/unspecified namespace is simply one more valid
/// namespace). Zero-length namespace string is assumed equal to unspecified
/// namespace.
pub const SERVICE_NAMESPACE: Key = Key::from_static_str("service.namespace");

/// The string ID of the service instance.
///
/// MUST be unique for each instance of the same
/// `service.namespace,service.name` pair (in other words
/// `service.namespace,service.name,service.instance.id` triplet MUST be
/// globally unique).
///
/// The ID helps to distinguish instances of the same service that exist at the
/// same time (e.g. instances of a horizontally scaled service). It is
/// preferable for the ID to be persistent and stay the same for the lifetime of
/// the service instance, however it is acceptable that the ID is ephemeral and
/// changes during important lifetime events for the service (e.g. service
/// restarts).
///
/// If the service has no inherent unique ID that can be used as the value of
/// this attribute it is recommended to generate a random Version 1 or Version 4
/// RFC 4122 UUID (services aiming for reproducible UUIDs may also use Version
/// 5, see RFC 4122 for more recommendations).
pub const SERVICE_INSTANCE_ID: Key = Key::from_static_str("service.instance.id");

/// The version string of the service API or implementation as defined in
/// [Version Attributes].
///
/// [Version Attributes]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions#version-attributes
pub const SERVICE_VERSION: Key = Key::from_static_str("service.version");

/// The name of the telemetry SDK as defined above.
pub const TELEMETRY_SDK_NAME: Key = Key::from_static_str("telemetry.sdk.name");

/// The language of the telemetry SDK.
/// One of the following values MUST be used, if one applies: "cpp", "dotnet",
/// "erlang", "go", "java", "nodejs", "php", "python", "ruby", "webjs"
pub const TELEMETRY_SDK_LANGUAGE: Key = Key::from_static_str("telemetry.sdk.language");

/// The version string of the telemetry SDK as defined in [Version Attributes].
///
/// [Version Attributes]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions#version-attributes
pub const TELEMETRY_SDK_VERSION: Key = Key::from_static_str("telemetry.sdk.version");

/// The version string of the auto instrumentation agent, if used, as defined in
/// [Version Attributes].
///
/// [Version Attributes]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions#version-attributes
pub const TELEMETRY_AUTO_VERSION: Key = Key::from_static_str("telemetry.auto.version");

/// Name of the cloud provider.
///
/// Example values are aws, azure, gcp.
pub const CLOUD_PROVIDER: Key = Key::from_static_str("cloud.provider");

/// The cloud account id used to identify different entities.
pub const CLOUD_ACCOUNT_ID: Key = Key::from_static_str("cloud.account.id");

/// A specific geographical location where different entities can run
pub const CLOUD_REGION: Key = Key::from_static_str("cloud.region");

/// Zones are a sub set of the region connected through low-latency links.
/// In aws it is called availability-zone.
pub const CLOUD_ZONE: Key = Key::from_static_str("cloud.zone");

/// Container name.
pub const CONTAINER_NAME: Key = Key::from_static_str("container.name");

/// Container id. Usually a UUID, as for example used to [identify Docker
/// containers]. The UUID might be abbreviated.
///
/// [identify Docker containers]: https://docs.docker.com/engine/reference/run/#container-identification
pub const CONTAINER_ID: Key = Key::from_static_str("container.id");

/// Name of the image the container was built on.
pub const CONTAINER_IMAGE_NAME: Key = Key::from_static_str("container.image.name");

/// Container image tag.
pub const CONTAINER_IMAGE_TAG: Key = Key::from_static_str("container.image.tag");

/// The name of the function being executed.
pub const FAAS_NAME: Key = Key::from_static_str("faas.name");

/// The unique name of the function being executed.
///
/// For example, in AWS Lambda this field corresponds to the [ARN] value, in GCP
/// to the URI of the resource, and in Azure to the [FunctionDirectory] field.
///
/// [ARN]: https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html
/// [FunctionDirectory]: https://github.com/Azure/azure-functions-host/wiki/Retrieving-information-about-the-currently-running-function
pub const FAAS_ID: Key = Key::from_static_str("faas.id");

/// The version string of the function being executed as defined in [Version
/// Attributes]
///
/// [Version Attributes]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions#version-attributes
pub const FAAS_VERSION: Key = Key::from_static_str("faas.version");

/// The execution environment ID as a string.
pub const FAAS_INSTANCE: Key = Key::from_static_str("faas.instance");

/// Hostname of the host.
///
/// It contains what the `hostname` command returns on the host machine.
pub const HOST_HOSTNAME: Key = Key::from_static_str("host.hostname");

/// Unique host id.
///
/// For Cloud this must be the instance_id assigned by the cloud provider
pub const HOST_ID: Key = Key::from_static_str("host.id");

/// Name of the host.
///
/// It may contain what `hostname` returns on Unix systems, the fully qualified,
/// or a name specified by the user.
pub const HOST_NAME: Key = Key::from_static_str("host.name");

/// Name of the VM image or OS install the host was instantiated from.
pub const HOST_IMAGE_NAME: Key = Key::from_static_str("host.image.name");

/// VM image id. For Cloud, this value is from the provider.
pub const HOST_IMAGE_ID: Key = Key::from_static_str("host.image.id");

/// The version string of the VM image as defined in [Version Attributes].
///
/// [Version Attributes]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions#version-attributes
pub const HOST_IMAGE_VERSION: Key = Key::from_static_str("host.image.version");

/// The name of the cluster.
pub const K8S_CLUSTER_NAME: Key = Key::from_static_str("k8s.cluster.name");

/// The name of the namespace that the pod is running in.
pub const K8S_NAMESPACE_NAME: Key = Key::from_static_str("k8s.namespace.name");

/// The uid of the Pod.
pub const K8S_POD_UID: Key = Key::from_static_str("k8s.pod.uid");

/// The name of the Pod.
pub const K8S_POD_NAME: Key = Key::from_static_str("k8s.pod.name");

/// The name of the Container in a Pod template.
pub const K8S_CONTAINER_NAME: Key = Key::from_static_str("k8s.container.name");

/// The uid of the ReplicaSet.
pub const K8S_REPLICASET_UID: Key = Key::from_static_str("k8s.replicaset.uid");

/// The name of the ReplicaSet.
pub const K8S_REPLICASET_NAME: Key = Key::from_static_str("k8s.replicaset.name");

/// The uid of the Deployment.
pub const K8S_DEPLOYMENT_UID: Key = Key::from_static_str("k8s.deployment.uid");

/// The name of the Deployment.
pub const K8S_DEPLOYMENT_NAME: Key = Key::from_static_str("k8s.deployment.name");

/// The uid of the StatefulSet.
pub const K8S_STATEFULSET_UID: Key = Key::from_static_str("k8s.statefulset.uid");

/// The name of the StatefulSet.
pub const K8S_STATEFULSET_NAME: Key = Key::from_static_str("k8s.statefulset.name");

/// The uid of the DaemonSet.
pub const K8S_DAEMONSET_UID: Key = Key::from_static_str("k8s.daemonset.uid");

/// The name of the DaemonSet.
pub const K8S_DAEMONSET_NAME: Key = Key::from_static_str("k8s.daemonset.name");

/// The uid of the Job.
pub const K8S_JOB_UID: Key = Key::from_static_str("k8s.job.uid");

/// The name of the Job.
pub const K8S_JOB_NAME: Key = Key::from_static_str("k8s.job.name");

/// The uid of the CronJob.
pub const K8S_CRONJOB_UID: Key = Key::from_static_str("k8s.cronjob.uid");

/// The name of the CronJob.
pub const K8S_CRONJOB_NAME: Key = Key::from_static_str("k8s.cronjob.name");

/// The operating system type.
pub const OS_TYPE: Key = Key::from_static_str("os.type");

/// Human readable (not intended to be parsed) OS version information, like e.g.
/// reported by `ver` or `lsb_release -a` commands.
pub const OS_DESCRIPTION: Key = Key::from_static_str("os.description");

/// Process identifier (PID).
pub const PROCESS_PID: Key = Key::from_static_str("process.pid");

/// The name of the process executable. On Linux based systems, can be set to
/// the `Name` in `proc/[pid]/status`. On Windows, can be set to the base name
/// of `GetProcessImageFileNameW`.
pub const PROCESS_EXECUTABLE_NAME: Key = Key::from_static_str("process.executable.name");

/// The full path to the process executable. On Linux based systems, can be set
/// to the target of `proc/[pid]/exe`. On Windows, can be set to the result of
/// `GetProcessImageFileNameW`.
pub const PROCESS_EXECUTABLE_PATH: Key = Key::from_static_str("process.executable.path");

/// The command used to launch the process (i.e. the command name). On Linux
/// based systems, can be set to the zeroth string in `proc/[pid]/cmdline`. On
/// Windows, can be set to the first parameter extracted from `GetCommandLineW`.
pub const PROCESS_COMMAND: Key = Key::from_static_str("process.command");

/// The full command used to launch the process. The value can be either a list
/// of strings representing the ordered list of arguments, or a single string
/// representing the full command. On Linux based systems, can be set to the
/// list of null-delimited strings extracted from `proc/[pid]/cmdline`. On
/// Windows, can be set to the result of `GetCommandLineW`.
pub const PROCESS_COMMAND_LINE: Key = Key::from_static_str("process.command_line");

/// The username of the user that owns the process.
pub const PROCESS_OWNER: Key = Key::from_static_str("process.owner");
