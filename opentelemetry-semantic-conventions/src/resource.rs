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
//! ```
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

/// The ID of a running ECS task. The ID MUST be extracted from `task.arn`.
///
/// # Examples
///
/// - `10838bed-421f-43ef-870a-f43feacbbb5b`
/// - `23ebb8ac-c18f-46c6-8bbe-d55d0e37cfbd`
pub const AWS_ECS_TASK_ID: &str = "aws.ecs.task.id";

/// The ARN of an [ECS cluster](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/clusters.html).
///
/// # Examples
///
/// - `arn:aws:ecs:us-west-2:123456789123:cluster/my-cluster`
pub const AWS_ECS_CLUSTER_ARN: &str = "aws.ecs.cluster.arn";

/// The Amazon Resource Name (ARN) of an [ECS container instance](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ECS_instances.html).
///
/// # Examples
///
/// - `arn:aws:ecs:us-west-1:123456789123:container/32624152-9086-4f0e-acae-1a75b14fe4d9`
pub const AWS_ECS_CONTAINER_ARN: &str = "aws.ecs.container.arn";

/// The [launch type](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/launch_types.html) for an ECS task.
pub const AWS_ECS_LAUNCHTYPE: &str = "aws.ecs.launchtype";

/// The ARN of a running [ECS task](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ecs-account-settings.html#ecs-resource-ids).
///
/// # Examples
///
/// - `arn:aws:ecs:us-west-1:123456789123:task/10838bed-421f-43ef-870a-f43feacbbb5b`
/// - `arn:aws:ecs:us-west-1:123456789123:task/my-cluster/task-id/23ebb8ac-c18f-46c6-8bbe-d55d0e37cfbd`
pub const AWS_ECS_TASK_ARN: &str = "aws.ecs.task.arn";

/// The family name of the [ECS task definition](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_definitions.html) used to create the ECS task.
///
/// # Examples
///
/// - `opentelemetry-family`
pub const AWS_ECS_TASK_FAMILY: &str = "aws.ecs.task.family";

/// The revision for the task definition used to create the ECS task.
///
/// # Examples
///
/// - `8`
/// - `26`
pub const AWS_ECS_TASK_REVISION: &str = "aws.ecs.task.revision";

/// The ARN of an EKS cluster.
///
/// # Examples
///
/// - `arn:aws:ecs:us-west-2:123456789123:cluster/my-cluster`
pub const AWS_EKS_CLUSTER_ARN: &str = "aws.eks.cluster.arn";

/// The Amazon Resource Name(s) (ARN) of the AWS log group(s).
///
/// See the [log group ARN format documentation](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/iam-access-control-overview-cwl.html#CWL_ARN_Format).
///
/// # Examples
///
/// - `arn:aws:logs:us-west-1:123456789012:log-group:/aws/my/group:*`
pub const AWS_LOG_GROUP_ARNS: &str = "aws.log.group.arns";

/// The name(s) of the AWS log group(s) an application is writing to.
///
/// Multiple log groups must be supported for cases like multi-container applications, where a single application has sidecar containers, and each write to their own log group.
///
/// # Examples
///
/// - `/aws/lambda/my-function`
/// - `opentelemetry-service`
pub const AWS_LOG_GROUP_NAMES: &str = "aws.log.group.names";

/// The ARN(s) of the AWS log stream(s).
///
/// See the [log stream ARN format documentation](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/iam-access-control-overview-cwl.html#CWL_ARN_Format). One log group can contain several log streams, so these ARNs necessarily identify both a log group and a log stream.
///
/// # Examples
///
/// - `arn:aws:logs:us-west-1:123456789012:log-group:/aws/my/group:log-stream:logs/main/10838bed-421f-43ef-870a-f43feacbbb5b`
pub const AWS_LOG_STREAM_ARNS: &str = "aws.log.stream.arns";

/// The name(s) of the AWS log stream(s) an application is writing to.
///
/// # Examples
///
/// - `logs/main/10838bed-421f-43ef-870a-f43feacbbb5b`
pub const AWS_LOG_STREAM_NAMES: &str = "aws.log.stream.names";

/// Unique identifier for the application.
///
/// # Examples
///
/// - `2daa2797-e42b-4624-9322-ec3f968df4da`
pub const HEROKU_APP_ID: &str = "heroku.app.id";

/// Commit hash for the current release.
///
/// # Examples
///
/// - `e6134959463efd8966b20e75b913cafe3f5ec`
pub const HEROKU_RELEASE_COMMIT: &str = "heroku.release.commit";

/// Time and date the release was created.
///
/// # Examples
///
/// - `2022-10-23T18:00:42Z`
pub const HEROKU_RELEASE_CREATION_TIMESTAMP: &str = "heroku.release.creation_timestamp";

/// The name of the web engine.
///
/// # Examples
///
/// - `WildFly`
pub const WEBENGINE_NAME: &str = "webengine.name";

/// Additional description of the web engine (e.g. detailed version and edition information).
///
/// # Examples
///
/// - `WildFly Full 21.0.0.Final (WildFly Core 13.0.1.Final) - 2.2.2.Final`
pub const WEBENGINE_DESCRIPTION: &str = "webengine.description";

/// The version of the web engine.
///
/// # Examples
///
/// - `21.0.0`
pub const WEBENGINE_VERSION: &str = "webengine.version";

/// The name of the instrumentation scope - (`InstrumentationScope.Name` in OTLP).
///
/// # Examples
///
/// - `io.opentelemetry.contrib.mongodb`
pub const OTEL_SCOPE_NAME: &str = "otel.scope.name";

/// The version of the instrumentation scope - (`InstrumentationScope.Version` in OTLP).
///
/// # Examples
///
/// - `1.0.0`
pub const OTEL_SCOPE_VERSION: &str = "otel.scope.version";

/// None.
///
/// # Examples
///
/// - `io.opentelemetry.contrib.mongodb`
pub const OTEL_LIBRARY_NAME: &str = "otel.library.name";

/// None.
///
/// # Examples
///
/// - `1.0.0`
pub const OTEL_LIBRARY_VERSION: &str = "otel.library.version";
