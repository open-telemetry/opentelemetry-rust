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
//! [resource semantic conventions]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/resource/semantic_conventions
//!
//! ## Usage
//!
//! ```rust,no_run
//! use opentelemetry::sdk;
//! use opentelemetry_semantic_conventions as semcov;
//!
//! let _tracer = opentelemetry::sdk::export::trace::stdout::new_pipeline()
//!     .with_trace_config(sdk::trace::config().with_resource(sdk::Resource::new(vec![
//!         semcov::resource::SERVICE_NAME.string("my-service"),
//!         semcov::resource::SERVICE_NAMESPACE.string("my-namespace"),
//!     ])))
//!     .install_simple();
//! ```

use opentelemetry::Key;

/// Name of the cloud provider.
///
/// # Examples
///
/// - gcp
pub const CLOUD_PROVIDER: Key = Key::from_static_str("cloud.provider");

/// The cloud account ID the resource is assigned to.
///
/// # Examples
///
/// - 111111111111
/// - opentelemetry
pub const CLOUD_ACCOUNT_ID: Key = Key::from_static_str("cloud.account.id");

/// The geographical region the resource is running. Refer to your provider&#39;s docs to see the available regions, for example [AWS regions](https://aws.amazon.com/about-aws/global-infrastructure/regions_az/), [Azure regions](https://azure.microsoft.com/en-us/global-infrastructure/geographies/), or [Google Cloud regions](https://cloud.google.com/about/locations).
///
/// # Examples
///
/// - us-central1
/// - us-east-1
pub const CLOUD_REGION: Key = Key::from_static_str("cloud.region");

/// Cloud regions often have multiple, isolated locations known as zones to increase availability. Availability zone represents the zone where the resource is running.
///
/// Availability zones are called &#34;zones&#34; on Google Cloud.
///
/// # Examples
///
/// - us-east-1c
pub const CLOUD_AVAILABILITY_ZONE: Key = Key::from_static_str("cloud.availability_zone");

/// The cloud platform in use.
///
/// The prefix of the service SHOULD match the one specified in `cloud.provider`.
///
/// # Examples
///
/// - aws_ec2
/// - azure_vm
/// - gcp_compute_engine
pub const CLOUD_PLATFORM: Key = Key::from_static_str("cloud.platform");

/// The Amazon Resource Name (ARN) of an [ECS container instance](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ECS_instances.html).
///
/// # Examples
///
/// - arn:aws:ecs:us-west-1:123456789123:container/32624152-9086-4f0e-acae-1a75b14fe4d9
pub const AWS_ECS_CONTAINER_ARN: Key = Key::from_static_str("aws.ecs.container.arn");

/// The ARN of an [ECS cluster](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/clusters.html).
///
/// # Examples
///
/// - arn:aws:ecs:us-west-2:123456789123:cluster/my-cluster
pub const AWS_ECS_CLUSTER_ARN: Key = Key::from_static_str("aws.ecs.cluster.arn");

/// The [launch type](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/launch_types.html) for an ECS task.
///
/// # Examples
///
/// - ec2
/// - fargate
pub const AWS_ECS_LAUNCHTYPE: Key = Key::from_static_str("aws.ecs.launchtype");

/// The ARN of an [ECS task definition](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_definitions.html).
///
/// # Examples
///
/// - arn:aws:ecs:us-west-1:123456789123:task/10838bed-421f-43ef-870a-f43feacbbb5b
pub const AWS_ECS_TASK_ARN: Key = Key::from_static_str("aws.ecs.task.arn");

/// The task definition family this task definition is a member of.
///
/// # Examples
///
/// - opentelemetry-family
pub const AWS_ECS_TASK_FAMILY: Key = Key::from_static_str("aws.ecs.task.family");

/// The revision for this task definition.
///
/// # Examples
///
/// - 8
/// - 26
pub const AWS_ECS_TASK_REVISION: Key = Key::from_static_str("aws.ecs.task.revision");

/// The ARN of an EKS cluster.
///
/// # Examples
///
/// - arn:aws:ecs:us-west-2:123456789123:cluster/my-cluster
pub const AWS_EKS_CLUSTER_ARN: Key = Key::from_static_str("aws.eks.cluster.arn");

/// The name(s) of the AWS log group(s) an application is writing to.
///
/// Multiple log groups must be supported for cases like multi-container applications, where a single application has sidecar containers, and each write to their own log group.
///
/// # Examples
///
/// - /aws/lambda/my-function
/// - opentelemetry-service
pub const AWS_LOG_GROUP_NAMES: Key = Key::from_static_str("aws.log.group.names");

/// The Amazon Resource Name(s) (ARN) of the AWS log group(s).
///
/// See the [log group ARN format documentation](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/iam-access-control-overview-cwl.html#CWL_ARN_Format).
///
/// # Examples
///
/// - arn:aws:logs:us-west-1:123456789012:log-group:/aws/my/group:*
pub const AWS_LOG_GROUP_ARNS: Key = Key::from_static_str("aws.log.group.arns");

/// The name(s) of the AWS log stream(s) an application is writing to.
///
/// # Examples
///
/// - logs/main/10838bed-421f-43ef-870a-f43feacbbb5b
pub const AWS_LOG_STREAM_NAMES: Key = Key::from_static_str("aws.log.stream.names");

/// The ARN(s) of the AWS log stream(s).
///
/// See the [log stream ARN format documentation](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/iam-access-control-overview-cwl.html#CWL_ARN_Format). One log group can contain several log streams, so these ARNs necessarily identify both a log group and a log stream.
///
/// # Examples
///
/// - arn:aws:logs:us-west-1:123456789012:log-group:/aws/my/group:log-stream:logs/main/10838bed-421f-43ef-870a-f43feacbbb5b
pub const AWS_LOG_STREAM_ARNS: Key = Key::from_static_str("aws.log.stream.arns");

/// Container name.
///
/// # Examples
///
/// - opentelemetry-autoconf
pub const CONTAINER_NAME: Key = Key::from_static_str("container.name");

/// Container ID. Usually a UUID, as for example used to [identify Docker containers](https://docs.docker.com/engine/reference/run/#container-identification). The UUID might be abbreviated.
///
/// # Examples
///
/// - a3bf90e006b2
pub const CONTAINER_ID: Key = Key::from_static_str("container.id");

/// The container runtime managing this container.
///
/// # Examples
///
/// - docker
/// - containerd
/// - rkt
pub const CONTAINER_RUNTIME: Key = Key::from_static_str("container.runtime");

/// Name of the image the container was built on.
///
/// # Examples
///
/// - gcr.io/opentelemetry/operator
pub const CONTAINER_IMAGE_NAME: Key = Key::from_static_str("container.image.name");

/// Container image tag.
///
/// # Examples
///
/// - 0.1
pub const CONTAINER_IMAGE_TAG: Key = Key::from_static_str("container.image.tag");

/// Name of the [deployment environment](https://en.wikipedia.org/wiki/Deployment_environment) (aka deployment tier).
///
/// # Examples
///
/// - staging
/// - production
pub const DEPLOYMENT_ENVIRONMENT: Key = Key::from_static_str("deployment.environment");

/// The name of the function being executed.
///
/// # Examples
///
/// - my-function
pub const FAAS_NAME: Key = Key::from_static_str("faas.name");

/// The unique ID of the function being executed.
///
/// For example, in AWS Lambda this field corresponds to the [ARN](https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html) value, in GCP to the URI of the resource, and in Azure to the [FunctionDirectory](https://github.com/Azure/azure-functions-host/wiki/Retrieving-information-about-the-currently-running-function) field.
///
/// # Examples
///
/// - arn:aws:lambda:us-west-2:123456789012:function:my-function
pub const FAAS_ID: Key = Key::from_static_str("faas.id");

/// The version string of the function being executed as defined in [Version Attributes](../../resource/semantic_conventions/README.md#version-attributes).
///
/// # Examples
///
/// - 2.0.0
pub const FAAS_VERSION: Key = Key::from_static_str("faas.version");

/// The execution environment ID as a string.
///
/// # Examples
///
/// - my-function:instance-0001
pub const FAAS_INSTANCE: Key = Key::from_static_str("faas.instance");

/// The amount of memory available to the serverless function in MiB.
///
/// It&#39;s recommended to set this attribute since e.g. too little memory can easily stop a Java AWS Lambda function from working correctly. On AWS Lambda, the environment variable `AWS_LAMBDA_FUNCTION_MEMORY_SIZE` provides this information.
///
/// # Examples
///
/// - 128
pub const FAAS_MAX_MEMORY: Key = Key::from_static_str("faas.max_memory");

/// Unique host ID. For Cloud, this must be the instance_id assigned by the cloud provider.
///
/// # Examples
///
/// - opentelemetry-test
pub const HOST_ID: Key = Key::from_static_str("host.id");

/// Name of the host. On Unix systems, it may contain what the hostname command returns, or the fully qualified hostname, or another name specified by the user.
///
/// # Examples
///
/// - opentelemetry-test
pub const HOST_NAME: Key = Key::from_static_str("host.name");

/// Type of host. For Cloud, this must be the machine type.
///
/// # Examples
///
/// - n1-standard-1
pub const HOST_TYPE: Key = Key::from_static_str("host.type");

/// The CPU architecture the host system is running on.
pub const HOST_ARCH: Key = Key::from_static_str("host.arch");

/// Name of the VM image or OS install the host was instantiated from.
///
/// # Examples
///
/// - infra-ami-eks-worker-node-7d4ec78312
/// - CentOS-8-x86_64-1905
pub const HOST_IMAGE_NAME: Key = Key::from_static_str("host.image.name");

/// VM image ID. For Cloud, this value is from the provider.
///
/// # Examples
///
/// - ami-07b06b442921831e5
pub const HOST_IMAGE_ID: Key = Key::from_static_str("host.image.id");

/// The version string of the VM image as defined in [Version Attributes](README.md#version-attributes).
///
/// # Examples
///
/// - 0.1
pub const HOST_IMAGE_VERSION: Key = Key::from_static_str("host.image.version");

/// The name of the cluster.
///
/// # Examples
///
/// - opentelemetry-cluster
pub const K8S_CLUSTER_NAME: Key = Key::from_static_str("k8s.cluster.name");

/// The name of the Node.
///
/// # Examples
///
/// - node-1
pub const K8S_NODE_NAME: Key = Key::from_static_str("k8s.node.name");

/// The UID of the Node.
///
/// # Examples
///
/// - 1eb3a0c6-0477-4080-a9cb-0cb7db65c6a2
pub const K8S_NODE_UID: Key = Key::from_static_str("k8s.node.uid");

/// The name of the namespace that the pod is running in.
///
/// # Examples
///
/// - default
pub const K8S_NAMESPACE_NAME: Key = Key::from_static_str("k8s.namespace.name");

/// The UID of the Pod.
///
/// # Examples
///
/// - 275ecb36-5aa8-4c2a-9c47-d8bb681b9aff
pub const K8S_POD_UID: Key = Key::from_static_str("k8s.pod.uid");

/// The name of the Pod.
///
/// # Examples
///
/// - opentelemetry-pod-autoconf
pub const K8S_POD_NAME: Key = Key::from_static_str("k8s.pod.name");

/// The name of the Container in a Pod template.
///
/// # Examples
///
/// - redis
pub const K8S_CONTAINER_NAME: Key = Key::from_static_str("k8s.container.name");

/// The UID of the ReplicaSet.
///
/// # Examples
///
/// - 275ecb36-5aa8-4c2a-9c47-d8bb681b9aff
pub const K8S_REPLICASET_UID: Key = Key::from_static_str("k8s.replicaset.uid");

/// The name of the ReplicaSet.
///
/// # Examples
///
/// - opentelemetry
pub const K8S_REPLICASET_NAME: Key = Key::from_static_str("k8s.replicaset.name");

/// The UID of the Deployment.
///
/// # Examples
///
/// - 275ecb36-5aa8-4c2a-9c47-d8bb681b9aff
pub const K8S_DEPLOYMENT_UID: Key = Key::from_static_str("k8s.deployment.uid");

/// The name of the Deployment.
///
/// # Examples
///
/// - opentelemetry
pub const K8S_DEPLOYMENT_NAME: Key = Key::from_static_str("k8s.deployment.name");

/// The UID of the StatefulSet.
///
/// # Examples
///
/// - 275ecb36-5aa8-4c2a-9c47-d8bb681b9aff
pub const K8S_STATEFULSET_UID: Key = Key::from_static_str("k8s.statefulset.uid");

/// The name of the StatefulSet.
///
/// # Examples
///
/// - opentelemetry
pub const K8S_STATEFULSET_NAME: Key = Key::from_static_str("k8s.statefulset.name");

/// The UID of the DaemonSet.
///
/// # Examples
///
/// - 275ecb36-5aa8-4c2a-9c47-d8bb681b9aff
pub const K8S_DAEMONSET_UID: Key = Key::from_static_str("k8s.daemonset.uid");

/// The name of the DaemonSet.
///
/// # Examples
///
/// - opentelemetry
pub const K8S_DAEMONSET_NAME: Key = Key::from_static_str("k8s.daemonset.name");

/// The UID of the Job.
///
/// # Examples
///
/// - 275ecb36-5aa8-4c2a-9c47-d8bb681b9aff
pub const K8S_JOB_UID: Key = Key::from_static_str("k8s.job.uid");

/// The name of the Job.
///
/// # Examples
///
/// - opentelemetry
pub const K8S_JOB_NAME: Key = Key::from_static_str("k8s.job.name");

/// The UID of the CronJob.
///
/// # Examples
///
/// - 275ecb36-5aa8-4c2a-9c47-d8bb681b9aff
pub const K8S_CRONJOB_UID: Key = Key::from_static_str("k8s.cronjob.uid");

/// The name of the CronJob.
///
/// # Examples
///
/// - opentelemetry
pub const K8S_CRONJOB_NAME: Key = Key::from_static_str("k8s.cronjob.name");

/// The operating system type.
pub const OS_TYPE: Key = Key::from_static_str("os.type");

/// Human readable (not intended to be parsed) OS version information, like e.g. reported by `ver` or `lsb_release -a` commands.
///
/// # Examples
///
/// - Microsoft Windows [Version 10.0.18363.778]
/// - Ubuntu 18.04.1 LTS
pub const OS_DESCRIPTION: Key = Key::from_static_str("os.description");

/// Process identifier (PID).
///
/// # Examples
///
/// - 1234
pub const PROCESS_PID: Key = Key::from_static_str("process.pid");

/// The name of the process executable. On Linux based systems, can be set to the `Name` in `proc/[pid]/status`. On Windows, can be set to the base name of `GetProcessImageFileNameW`.
///
/// # Examples
///
/// - otelcol
pub const PROCESS_EXECUTABLE_NAME: Key = Key::from_static_str("process.executable.name");

/// The full path to the process executable. On Linux based systems, can be set to the target of `proc/[pid]/exe`. On Windows, can be set to the result of `GetProcessImageFileNameW`.
///
/// # Examples
///
/// - /usr/bin/cmd/otelcol
pub const PROCESS_EXECUTABLE_PATH: Key = Key::from_static_str("process.executable.path");

/// The command used to launch the process (i.e. the command name). On Linux based systems, can be set to the zeroth string in `proc/[pid]/cmdline`. On Windows, can be set to the first parameter extracted from `GetCommandLineW`.
///
/// # Examples
///
/// - cmd/otelcol
pub const PROCESS_COMMAND: Key = Key::from_static_str("process.command");

/// The full command used to launch the process as a single string representing the full command. On Windows, can be set to the result of `GetCommandLineW`. Do not set this if you have to assemble it just for monitoring; use `process.command_args` instead.
///
/// # Examples
///
/// - C:\cmd\otecol --config=&#34;my directory\config.yaml&#34;
pub const PROCESS_COMMAND_LINE: Key = Key::from_static_str("process.command_line");

/// All the command arguments (including the command/executable itself) as received by the process. On Linux-based systems (and some other Unixoid systems supporting procfs), can be set according to the list of null-delimited strings extracted from `proc/[pid]/cmdline`. For libc-based executables, this would be the full argv vector passed to `main`.
///
/// # Examples
///
/// - cmd/otecol
/// - --config=config.yaml
pub const PROCESS_COMMAND_ARGS: Key = Key::from_static_str("process.command_args");

/// The username of the user that owns the process.
///
/// # Examples
///
/// - root
pub const PROCESS_OWNER: Key = Key::from_static_str("process.owner");

/// The name of the runtime of this process. For compiled native binaries, this SHOULD be the name of the compiler.
///
/// # Examples
///
/// - OpenJDK Runtime Environment
pub const PROCESS_RUNTIME_NAME: Key = Key::from_static_str("process.runtime.name");

/// The version of the runtime of this process, as returned by the runtime without modification.
///
/// # Examples
///
/// - 14.0.2
pub const PROCESS_RUNTIME_VERSION: Key = Key::from_static_str("process.runtime.version");

/// An additional description about the runtime of the process, for example a specific vendor customization of the runtime environment.
///
/// # Examples
///
/// - Eclipse OpenJ9 Eclipse OpenJ9 VM openj9-0.21.0
pub const PROCESS_RUNTIME_DESCRIPTION: Key = Key::from_static_str("process.runtime.description");

/// Logical name of the service.
///
/// MUST be the same for all instances of horizontally scaled services. If the value was not specified, SDKs MUST fallback to `unknown_service:` concatenated with [`process.executable.name`](process.md#process), e.g. `unknown_service:bash`. If `process.executable.name` is not available, the value MUST be set to `unknown_service`.
///
/// # Examples
///
/// - shoppingcart
pub const SERVICE_NAME: Key = Key::from_static_str("service.name");

/// A namespace for `service.name`.
///
/// A string value having a meaning that helps to distinguish a group of services, for example the team name that owns a group of services. `service.name` is expected to be unique within the same namespace. If `service.namespace` is not specified in the Resource then `service.name` is expected to be unique for all services that have no explicit namespace defined (so the empty/unspecified namespace is simply one more valid namespace). Zero-length namespace string is assumed equal to unspecified namespace.
///
/// # Examples
///
/// - Shop
pub const SERVICE_NAMESPACE: Key = Key::from_static_str("service.namespace");

/// The string ID of the service instance.
///
/// MUST be unique for each instance of the same `service.namespace,service.name` pair (in other words `service.namespace,service.name,service.instance.id` triplet MUST be globally unique). The ID helps to distinguish instances of the same service that exist at the same time (e.g. instances of a horizontally scaled service). It is preferable for the ID to be persistent and stay the same for the lifetime of the service instance, however it is acceptable that the ID is ephemeral and changes during important lifetime events for the service (e.g. service restarts). If the service has no inherent unique ID that can be used as the value of this attribute it is recommended to generate a random Version 1 or Version 4 RFC 4122 UUID (services aiming for reproducible UUIDs may also use Version 5, see RFC 4122 for more recommendations).
///
/// # Examples
///
/// - 627cc493-f310-47de-96bd-71410b7dec09
pub const SERVICE_INSTANCE_ID: Key = Key::from_static_str("service.instance.id");

/// The version string of the service API or implementation.
///
/// # Examples
///
/// - 2.0.0
pub const SERVICE_VERSION: Key = Key::from_static_str("service.version");

/// The name of the telemetry SDK as defined above.
///
/// # Examples
///
/// - opentelemetry
pub const TELEMETRY_SDK_NAME: Key = Key::from_static_str("telemetry.sdk.name");

/// The language of the telemetry SDK.
pub const TELEMETRY_SDK_LANGUAGE: Key = Key::from_static_str("telemetry.sdk.language");

/// The version string of the telemetry SDK.
///
/// # Examples
///
/// - 1.2.3
pub const TELEMETRY_SDK_VERSION: Key = Key::from_static_str("telemetry.sdk.version");

/// The version string of the auto instrumentation agent, if used.
///
/// # Examples
///
/// - 1.2.3
pub const TELEMETRY_AUTO_VERSION: Key = Key::from_static_str("telemetry.auto.version");

/// The name of the web engine.
///
/// # Examples
///
/// - WildFly
pub const WEBENGINE_NAME: Key = Key::from_static_str("webengine.name");

/// The version of the web engine.
///
/// # Examples
///
/// - 21.0.0
pub const WEBENGINE_VERSION: Key = Key::from_static_str("webengine.version");

/// Additional description of the web engine (e.g. detailed version and edition information).
///
/// # Examples
///
/// - WildFly Full 21.0.0.Final (WildFly Core 13.0.1.Final) - 2.2.2.Final
pub const WEBENGINE_DESCRIPTION: Key = Key::from_static_str("webengine.description");
