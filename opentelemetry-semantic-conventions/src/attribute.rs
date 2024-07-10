// DO NOT EDIT, this is an auto-generated file
//
// If you want to update the file:
// - Edit the template at scripts/templates/semantic_attributes.rs.j2
// - Run the script at scripts/generate-consts-from-spec.sh

//! # Semantic Attributes
//!
//! The entire set of semantic attributes (or [conventions](https://opentelemetry.io/docs/concepts/semantic-conventions/)) defined by the project. The resource, metric, and trace modules reference these attributes.
/// Uniquely identifies the framework API revision offered by a version (`os.version`) of the android operating system. More information can be found [here](https://developer.android.com/guide/topics/manifest/uses-sdk-element#ApiLevels).
///
/// # Examples
///
/// - `33`
/// - `32`
pub const ANDROID_OS_API_LEVEL: &str = "android.os.api_level";
/// Deprecated use the `device.app.lifecycle` event definition including `android.state` as a payload field instead.
///
/// The Android lifecycle states are defined in [Activity lifecycle callbacks](https://developer.android.com/guide/components/activities/activity-lifecycle#lc), and from which the `OS identifiers` are derived.
pub const ANDROID_STATE: &str = "android.state";
/// ASP.NET Core exception middleware handling result.
///
/// # Examples
///
/// - `handled`
/// - `unhandled`
pub const ASPNETCORE_DIAGNOSTICS_EXCEPTION_RESULT: &str = "aspnetcore.diagnostics.exception.result";
/// Full type name of the [`IExceptionHandler`](https://learn.microsoft.com/dotnet/api/microsoft.aspnetcore.diagnostics.iexceptionhandler) implementation that handled the exception.
///
/// # Examples
///
/// - `Contoso.MyHandler`
pub const ASPNETCORE_DIAGNOSTICS_HANDLER_TYPE: &str = "aspnetcore.diagnostics.handler.type";
/// Rate limiting policy name.
///
/// # Examples
///
/// - `fixed`
/// - `sliding`
/// - `token`
pub const ASPNETCORE_RATE_LIMITING_POLICY: &str = "aspnetcore.rate_limiting.policy";
/// Rate-limiting result, shows whether the lease was acquired or contains a rejection reason.
///
/// # Examples
///
/// - `acquired`
/// - `request_canceled`
pub const ASPNETCORE_RATE_LIMITING_RESULT: &str = "aspnetcore.rate_limiting.result";
/// Flag indicating if request was handled by the application pipeline.
///
/// # Examples
///
/// - `True`
pub const ASPNETCORE_REQUEST_IS_UNHANDLED: &str = "aspnetcore.request.is_unhandled";
/// A value that indicates whether the matched route is a fallback route.
///
/// # Examples
///
/// - `True`
pub const ASPNETCORE_ROUTING_IS_FALLBACK: &str = "aspnetcore.routing.is_fallback";
/// Match result - success or failure.
///
/// # Examples
///
/// - `success`
/// - `failure`
pub const ASPNETCORE_ROUTING_MATCH_STATUS: &str = "aspnetcore.routing.match_status";
/// The JSON-serialized value of each item in the `AttributeDefinitions` request field.
///
/// # Examples
///
/// - `{ "AttributeName": "string", "AttributeType": "string" }`
pub const AWS_DYNAMODB_ATTRIBUTE_DEFINITIONS: &str = "aws.dynamodb.attribute_definitions";
/// The value of the `AttributesToGet` request parameter.
///
/// # Examples
///
/// - `lives`
/// - `id`
pub const AWS_DYNAMODB_ATTRIBUTES_TO_GET: &str = "aws.dynamodb.attributes_to_get";
/// The value of the `ConsistentRead` request parameter.
pub const AWS_DYNAMODB_CONSISTENT_READ: &str = "aws.dynamodb.consistent_read";
/// The JSON-serialized value of each item in the `ConsumedCapacity` response field.
///
/// # Examples
///
/// - `{ "CapacityUnits": number, "GlobalSecondaryIndexes": { "string" : { "CapacityUnits": number, "ReadCapacityUnits": number, "WriteCapacityUnits": number } }, "LocalSecondaryIndexes": { "string" : { "CapacityUnits": number, "ReadCapacityUnits": number, "WriteCapacityUnits": number } }, "ReadCapacityUnits": number, "Table": { "CapacityUnits": number, "ReadCapacityUnits": number, "WriteCapacityUnits": number }, "TableName": "string", "WriteCapacityUnits": number }`
pub const AWS_DYNAMODB_CONSUMED_CAPACITY: &str = "aws.dynamodb.consumed_capacity";
/// The value of the `Count` response parameter.
///
/// # Examples
///
/// - `10`
pub const AWS_DYNAMODB_COUNT: &str = "aws.dynamodb.count";
/// The value of the `ExclusiveStartTableName` request parameter.
///
/// # Examples
///
/// - `Users`
/// - `CatsTable`
pub const AWS_DYNAMODB_EXCLUSIVE_START_TABLE: &str = "aws.dynamodb.exclusive_start_table";
/// The JSON-serialized value of each item in the `GlobalSecondaryIndexUpdates` request field.
///
/// # Examples
///
/// - `{ "Create": { "IndexName": "string", "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" }, "ProvisionedThroughput": { "ReadCapacityUnits": number, "WriteCapacityUnits": number } }`
pub const AWS_DYNAMODB_GLOBAL_SECONDARY_INDEX_UPDATES: &str =
    "aws.dynamodb.global_secondary_index_updates";
/// The JSON-serialized value of each item of the `GlobalSecondaryIndexes` request field.
///
/// # Examples
///
/// - `{ "IndexName": "string", "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" }, "ProvisionedThroughput": { "ReadCapacityUnits": number, "WriteCapacityUnits": number } }`
pub const AWS_DYNAMODB_GLOBAL_SECONDARY_INDEXES: &str = "aws.dynamodb.global_secondary_indexes";
/// The value of the `IndexName` request parameter.
///
/// # Examples
///
/// - `name_to_group`
pub const AWS_DYNAMODB_INDEX_NAME: &str = "aws.dynamodb.index_name";
/// The JSON-serialized value of the `ItemCollectionMetrics` response field.
///
/// # Examples
///
/// - `{ "string" : [ { "ItemCollectionKey": { "string" : { "B": blob, "BOOL": boolean, "BS": [ blob ], "L": [ "AttributeValue" ], "M": { "string" : "AttributeValue" }, "N": "string", "NS": [ "string" ], "NULL": boolean, "S": "string", "SS": [ "string" ] } }, "SizeEstimateRangeGB": [ number ] } ] }`
pub const AWS_DYNAMODB_ITEM_COLLECTION_METRICS: &str = "aws.dynamodb.item_collection_metrics";
/// The value of the `Limit` request parameter.
///
/// # Examples
///
/// - `10`
pub const AWS_DYNAMODB_LIMIT: &str = "aws.dynamodb.limit";
/// The JSON-serialized value of each item of the `LocalSecondaryIndexes` request field.
///
/// # Examples
///
/// - `{ "IndexArn": "string", "IndexName": "string", "IndexSizeBytes": number, "ItemCount": number, "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" } }`
pub const AWS_DYNAMODB_LOCAL_SECONDARY_INDEXES: &str = "aws.dynamodb.local_secondary_indexes";
/// The value of the `ProjectionExpression` request parameter.
///
/// # Examples
///
/// - `Title`
/// - `Title, Price, Color`
/// - `Title, Description, RelatedItems, ProductReviews`
pub const AWS_DYNAMODB_PROJECTION: &str = "aws.dynamodb.projection";
/// The value of the `ProvisionedThroughput.ReadCapacityUnits` request parameter.
///
/// # Examples
///
/// - `1.0`
/// - `2.0`
pub const AWS_DYNAMODB_PROVISIONED_READ_CAPACITY: &str = "aws.dynamodb.provisioned_read_capacity";
/// The value of the `ProvisionedThroughput.WriteCapacityUnits` request parameter.
///
/// # Examples
///
/// - `1.0`
/// - `2.0`
pub const AWS_DYNAMODB_PROVISIONED_WRITE_CAPACITY: &str = "aws.dynamodb.provisioned_write_capacity";
/// The value of the `ScanIndexForward` request parameter.
pub const AWS_DYNAMODB_SCAN_FORWARD: &str = "aws.dynamodb.scan_forward";
/// The value of the `ScannedCount` response parameter.
///
/// # Examples
///
/// - `50`
pub const AWS_DYNAMODB_SCANNED_COUNT: &str = "aws.dynamodb.scanned_count";
/// The value of the `Segment` request parameter.
///
/// # Examples
///
/// - `10`
pub const AWS_DYNAMODB_SEGMENT: &str = "aws.dynamodb.segment";
/// The value of the `Select` request parameter.
///
/// # Examples
///
/// - `ALL_ATTRIBUTES`
/// - `COUNT`
pub const AWS_DYNAMODB_SELECT: &str = "aws.dynamodb.select";
/// The number of items in the `TableNames` response parameter.
///
/// # Examples
///
/// - `20`
pub const AWS_DYNAMODB_TABLE_COUNT: &str = "aws.dynamodb.table_count";
/// The keys in the `RequestItems` object field.
///
/// # Examples
///
/// - `Users`
/// - `Cats`
pub const AWS_DYNAMODB_TABLE_NAMES: &str = "aws.dynamodb.table_names";
/// The value of the `TotalSegments` request parameter.
///
/// # Examples
///
/// - `100`
pub const AWS_DYNAMODB_TOTAL_SEGMENTS: &str = "aws.dynamodb.total_segments";
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
/// The ID of a running ECS task. The ID MUST be extracted from `task.arn`.
///
/// # Examples
///
/// - `10838bed-421f-43ef-870a-f43feacbbb5b`
/// - `23ebb8ac-c18f-46c6-8bbe-d55d0e37cfbd`
pub const AWS_ECS_TASK_ID: &str = "aws.ecs.task.id";
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
/// The full invoked ARN as provided on the `Context` passed to the function (`Lambda-Runtime-Invoked-Function-Arn` header on the `/runtime/invocation/next` applicable).
///
/// This may be different from `cloud.resource_id` if an alias is involved.
///
/// # Examples
///
/// - `arn:aws:lambda:us-east-1:123456:function:myfunction:myalias`
pub const AWS_LAMBDA_INVOKED_ARN: &str = "aws.lambda.invoked_arn";
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
/// The AWS request ID as returned in the response headers `x-amz-request-id` or `x-amz-requestid`.
///
/// # Examples
///
/// - `79b9da39-b7ae-508a-a6bc-864b2829c622`
/// - `C9ER4AJX75574TDJ`
pub const AWS_REQUEST_ID: &str = "aws.request_id";
/// The S3 bucket name the request refers to. Corresponds to the `--bucket` parameter of the [S3 API](https://docs.aws.amazon.com/cli/latest/reference/s3api/index.html) operations.
///
/// The `bucket` attribute is applicable to all S3 operations that reference a bucket, i.e. that require the bucket name as a mandatory parameter.
/// This applies to almost all S3 operations except `list-buckets`.
///
/// # Examples
///
/// - `some-bucket-name`
pub const AWS_S3_BUCKET: &str = "aws.s3.bucket";
/// The source object (in the form `bucket`/`key`) for the copy operation.
///
/// The `copy_source` attribute applies to S3 copy operations and corresponds to the `--copy-source` parameter
/// of the [copy-object operation within the S3 API](https://docs.aws.amazon.com/cli/latest/reference/s3api/copy-object.html).
/// This applies in particular to the following operations:
///
/// - [copy-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/copy-object.html)
/// - [upload-part-copy](https://docs.aws.amazon.com/cli/latest/reference/s3api/upload-part-copy.html)
///
/// # Examples
///
/// - `someFile.yml`
pub const AWS_S3_COPY_SOURCE: &str = "aws.s3.copy_source";
/// The delete request container that specifies the objects to be deleted.
///
/// The `delete` attribute is only applicable to the [delete-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/delete-object.html) operation.
/// The `delete` attribute corresponds to the `--delete` parameter of the
/// [delete-objects operation within the S3 API](https://docs.aws.amazon.com/cli/latest/reference/s3api/delete-objects.html).
///
/// # Examples
///
/// - `Objects=[{Key=string,VersionId=string},{Key=string,VersionId=string}],Quiet=boolean`
pub const AWS_S3_DELETE: &str = "aws.s3.delete";
/// The S3 object key the request refers to. Corresponds to the `--key` parameter of the [S3 API](https://docs.aws.amazon.com/cli/latest/reference/s3api/index.html) operations.
///
/// The `key` attribute is applicable to all object-related S3 operations, i.e. that require the object key as a mandatory parameter.
/// This applies in particular to the following operations:
///
/// - [copy-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/copy-object.html)
/// - [delete-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/delete-object.html)
/// - [get-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/get-object.html)
/// - [head-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/head-object.html)
/// - [put-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/put-object.html)
/// - [restore-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/restore-object.html)
/// - [select-object-content](https://docs.aws.amazon.com/cli/latest/reference/s3api/select-object-content.html)
/// - [abort-multipart-upload](https://docs.aws.amazon.com/cli/latest/reference/s3api/abort-multipart-upload.html)
/// - [complete-multipart-upload](https://docs.aws.amazon.com/cli/latest/reference/s3api/complete-multipart-upload.html)
/// - [create-multipart-upload](https://docs.aws.amazon.com/cli/latest/reference/s3api/create-multipart-upload.html)
/// - [list-parts](https://docs.aws.amazon.com/cli/latest/reference/s3api/list-parts.html)
/// - [upload-part](https://docs.aws.amazon.com/cli/latest/reference/s3api/upload-part.html)
/// - [upload-part-copy](https://docs.aws.amazon.com/cli/latest/reference/s3api/upload-part-copy.html)
///
/// # Examples
///
/// - `someFile.yml`
pub const AWS_S3_KEY: &str = "aws.s3.key";
/// The part number of the part being uploaded in a multipart-upload operation. This is a positive integer between 1 and 10,000.
///
/// The `part_number` attribute is only applicable to the [upload-part](https://docs.aws.amazon.com/cli/latest/reference/s3api/upload-part.html)
/// and [upload-part-copy](https://docs.aws.amazon.com/cli/latest/reference/s3api/upload-part-copy.html) operations.
/// The `part_number` attribute corresponds to the `--part-number` parameter of the
/// [upload-part operation within the S3 API](https://docs.aws.amazon.com/cli/latest/reference/s3api/upload-part.html).
///
/// # Examples
///
/// - `3456`
pub const AWS_S3_PART_NUMBER: &str = "aws.s3.part_number";
/// Upload ID that identifies the multipart upload.
///
/// The `upload_id` attribute applies to S3 multipart-upload operations and corresponds to the `--upload-id` parameter
/// of the [S3 API](https://docs.aws.amazon.com/cli/latest/reference/s3api/index.html) multipart operations.
/// This applies in particular to the following operations:
///
/// - [abort-multipart-upload](https://docs.aws.amazon.com/cli/latest/reference/s3api/abort-multipart-upload.html)
/// - [complete-multipart-upload](https://docs.aws.amazon.com/cli/latest/reference/s3api/complete-multipart-upload.html)
/// - [list-parts](https://docs.aws.amazon.com/cli/latest/reference/s3api/list-parts.html)
/// - [upload-part](https://docs.aws.amazon.com/cli/latest/reference/s3api/upload-part.html)
/// - [upload-part-copy](https://docs.aws.amazon.com/cli/latest/reference/s3api/upload-part-copy.html)
///
/// # Examples
///
/// - `dfRtDYWFbkRONycy.Yxwh66Yjlx.cph0gtNBtJ`
pub const AWS_S3_UPLOAD_ID: &str = "aws.s3.upload_id";
/// Array of brand name and version separated by a space.
///
/// This value is intended to be taken from the [UA client hints API](https://wicg.github.io/ua-client-hints/#interface) (`navigator.userAgentData.brands`).
///
/// # Examples
///
/// - ` Not A;Brand 99`
/// - `Chromium 99`
/// - `Chrome 99`
pub const BROWSER_BRANDS: &str = "browser.brands";
/// Preferred language of the user using the browser.
///
/// This value is intended to be taken from the Navigator API `navigator.language`.
///
/// # Examples
///
/// - `en`
/// - `en-US`
/// - `fr`
/// - `fr-FR`
pub const BROWSER_LANGUAGE: &str = "browser.language";
/// A boolean that is true if the browser is running on a mobile device.
///
/// This value is intended to be taken from the [UA client hints API](https://wicg.github.io/ua-client-hints/#interface) (`navigator.userAgentData.mobile`). If unavailable, this attribute SHOULD be left unset.
pub const BROWSER_MOBILE: &str = "browser.mobile";
/// The platform on which the browser is running.
///
/// This value is intended to be taken from the [UA client hints API](https://wicg.github.io/ua-client-hints/#interface) (`navigator.userAgentData.platform`). If unavailable, the legacy `navigator.platform` API SHOULD NOT be used instead and this attribute SHOULD be left unset in order for the values to be consistent.
/// The list of possible values is defined in the [W3C User-Agent Client Hints specification](https://wicg.github.io/ua-client-hints/#sec-ch-ua-platform). Note that some (but not all) of these values can overlap with values in the [`os.type` and `os.name` attributes](./os.md). However, for consistency, the values in the `browser.platform` attribute should capture the exact value that the user agent provides.
///
/// # Examples
///
/// - `Windows`
/// - `macOS`
/// - `Android`
pub const BROWSER_PLATFORM: &str = "browser.platform";
/// Client address - domain name if available without reverse DNS lookup; otherwise, IP address or Unix domain socket name.
///
/// When observed from the server side, and when communicating through an intermediary, `client.address` SHOULD represent the client address behind any intermediaries,  for example proxies, if it&#39;s available.
///
/// # Examples
///
/// - `client.example.com`
/// - `10.1.2.80`
/// - `/tmp/my.sock`
pub const CLIENT_ADDRESS: &str = "client.address";
/// Client port number.
///
/// When observed from the server side, and when communicating through an intermediary, `client.port` SHOULD represent the client port behind any intermediaries,  for example proxies, if it&#39;s available.
///
/// # Examples
///
/// - `65123`
pub const CLIENT_PORT: &str = "client.port";
/// The cloud account ID the resource is assigned to.
///
/// # Examples
///
/// - `111111111111`
/// - `opentelemetry`
pub const CLOUD_ACCOUNT_ID: &str = "cloud.account.id";
/// Cloud regions often have multiple, isolated locations known as zones to increase availability. Availability zone represents the zone where the resource is running.
///
/// Availability zones are called &#34;zones&#34; on Alibaba Cloud and Google Cloud.
///
/// # Examples
///
/// - `us-east-1c`
pub const CLOUD_AVAILABILITY_ZONE: &str = "cloud.availability_zone";
/// The cloud platform in use.
///
/// The prefix of the service SHOULD match the one specified in `cloud.provider`.
pub const CLOUD_PLATFORM: &str = "cloud.platform";
/// Name of the cloud provider.
pub const CLOUD_PROVIDER: &str = "cloud.provider";
/// The geographical region the resource is running.
///
/// Refer to your provider&#39;s docs to see the available regions, for example [Alibaba Cloud regions](https://www.alibabacloud.com/help/doc-detail/40654.htm), [AWS regions](https://aws.amazon.com/about-aws/global-infrastructure/regions_az/), [Azure regions](https://azure.microsoft.com/global-infrastructure/geographies/), [Google Cloud regions](https://cloud.google.com/about/locations), or [Tencent Cloud regions](https://www.tencentcloud.com/document/product/213/6091).
///
/// # Examples
///
/// - `us-central1`
/// - `us-east-1`
pub const CLOUD_REGION: &str = "cloud.region";
/// Cloud provider-specific native identifier of the monitored cloud resource (e.g. an [ARN](https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html) on AWS, a [fully qualified resource ID](https://learn.microsoft.com/rest/api/resources/resources/get-by-id) on Azure, a [full resource name](https://cloud.google.com/apis/design/resource_names#full_resource_name) on GCP).
///
/// On some cloud providers, it may not be possible to determine the full ID at startup,
/// so it may be necessary to set `cloud.resource_id` as a span attribute instead.
///
/// The exact value to use for `cloud.resource_id` depends on the cloud provider.
/// The following well-known definitions MUST be used if you set this attribute and they apply:
///
/// * **AWS Lambda:** The function [ARN](https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html).
///   Take care not to use the &#34;invoked ARN&#34; directly but replace any
///   [alias suffix](https://docs.aws.amazon.com/lambda/latest/dg/configuration-aliases.html)
///   with the resolved function version, as the same runtime instance may be invokable with
///   multiple different aliases.
/// * **GCP:** The [URI of the resource](https://cloud.google.com/iam/docs/full-resource-names)
/// * **Azure:** The [Fully Qualified Resource ID](https://docs.microsoft.com/rest/api/resources/resources/get-by-id) of the invoked function,
///   *not* the function app, having the form
///   `/subscriptions/&lt;SUBSCIPTION_GUID&gt;/resourceGroups/&lt;RG&gt;/providers/Microsoft.Web/sites/&lt;FUNCAPP&gt;/functions/&lt;FUNC&gt;`.
///   This means that a span attribute MUST be used, as an Azure function app can host multiple functions that would usually share
///   a TracerProvider.
///
/// # Examples
///
/// - `arn:aws:lambda:REGION:ACCOUNT_ID:function:my-function`
/// - `//run.googleapis.com/projects/PROJECT_ID/locations/LOCATION_ID/services/SERVICE_ID`
/// - `/subscriptions/<SUBSCIPTION_GUID>/resourceGroups/<RG>/providers/Microsoft.Web/sites/<FUNCAPP>/functions/<FUNC>`
pub const CLOUD_RESOURCE_ID: &str = "cloud.resource_id";
/// The [event_id](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#id) uniquely identifies the event.
///
/// # Examples
///
/// - `123e4567-e89b-12d3-a456-426614174000`
/// - `0001`
pub const CLOUDEVENTS_EVENT_ID: &str = "cloudevents.event_id";
/// The [source](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#source-1) identifies the context in which an event happened.
///
/// # Examples
///
/// - `https://github.com/cloudevents`
/// - `/cloudevents/spec/pull/123`
/// - `my-service`
pub const CLOUDEVENTS_EVENT_SOURCE: &str = "cloudevents.event_source";
/// The [version of the CloudEvents specification](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#specversion) which the event uses.
///
/// # Examples
///
/// - `1.0`
pub const CLOUDEVENTS_EVENT_SPEC_VERSION: &str = "cloudevents.event_spec_version";
/// The [subject](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#subject) of the event in the context of the event producer (identified by source).
///
/// # Examples
///
/// - `mynewfile.jpg`
pub const CLOUDEVENTS_EVENT_SUBJECT: &str = "cloudevents.event_subject";
/// The [event_type](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#type) contains a value describing the type of event related to the originating occurrence.
///
/// # Examples
///
/// - `com.github.pull_request.opened`
/// - `com.example.object.deleted.v2`
pub const CLOUDEVENTS_EVENT_TYPE: &str = "cloudevents.event_type";
/// The column number in `code.filepath` best representing the operation. It SHOULD point within the code unit named in `code.function`.
///
/// # Examples
///
/// - `16`
pub const CODE_COLUMN: &str = "code.column";
/// The source code file name that identifies the code unit as uniquely as possible (preferably an absolute file path).
///
/// # Examples
///
/// - `/usr/local/MyApplication/content_root/app/index.php`
pub const CODE_FILEPATH: &str = "code.filepath";
/// The method or function name, or equivalent (usually rightmost part of the code unit&#39;s name).
///
/// # Examples
///
/// - `serveRequest`
pub const CODE_FUNCTION: &str = "code.function";
/// The line number in `code.filepath` best representing the operation. It SHOULD point within the code unit named in `code.function`.
///
/// # Examples
///
/// - `42`
pub const CODE_LINENO: &str = "code.lineno";
/// The &#34;namespace&#34; within which `code.function` is defined. Usually the qualified class or module name, such that `code.namespace` + some separator + `code.function` form a unique identifier for the code unit.
///
/// # Examples
///
/// - `com.example.MyHttpService`
pub const CODE_NAMESPACE: &str = "code.namespace";
/// A stacktrace as a string in the natural representation for the language runtime. The representation is to be determined and documented by each language SIG.
///
/// # Examples
///
/// - `at com.example.GenerateTrace.methodB(GenerateTrace.java:13)\n at com.example.GenerateTrace.methodA(GenerateTrace.java:9)\n at com.example.GenerateTrace.main(GenerateTrace.java:5)`
pub const CODE_STACKTRACE: &str = "code.stacktrace";
/// The command used to run the container (i.e. the command name).
///
/// If using embedded credentials or sensitive data, it is recommended to remove them to prevent potential leakage.
///
/// # Examples
///
/// - `otelcontribcol`
pub const CONTAINER_COMMAND: &str = "container.command";
/// All the command arguments (including the command/executable itself) run by the container.
///
/// # Examples
///
/// - `otelcontribcol, --config, config.yaml`
pub const CONTAINER_COMMAND_ARGS: &str = "container.command_args";
/// The full command run by the container as a single string representing the full command.
///
/// # Examples
///
/// - `otelcontribcol --config config.yaml`
pub const CONTAINER_COMMAND_LINE: &str = "container.command_line";
/// The CPU state for this data point. A container SHOULD be characterized _either_ by data points with no `state` labels, _or only_ data points with `state` labels.
///
/// # Examples
///
/// - `user`
/// - `kernel`
pub const CONTAINER_CPU_STATE: &str = "container.cpu.state";
/// Container ID. Usually a UUID, as for example used to [identify Docker containers](https://docs.docker.com/engine/reference/run/#container-identification). The UUID might be abbreviated.
///
/// # Examples
///
/// - `a3bf90e006b2`
pub const CONTAINER_ID: &str = "container.id";
/// Runtime specific image identifier. Usually a hash algorithm followed by a UUID.
///
/// Docker defines a sha256 of the image id; `container.image.id` corresponds to the `Image` field from the Docker container inspect [API](https://docs.docker.com/engine/api/v1.43/#tag/Container/operation/ContainerInspect) endpoint.
/// K8s defines a link to the container registry repository with digest `&#34;imageID&#34;: &#34;registry.azurecr.io /namespace/service/dockerfile@sha256:bdeabd40c3a8a492eaf9e8e44d0ebbb84bac7ee25ac0cf8a7159d25f62555625&#34;`.
/// The ID is assigned by the container runtime and can vary in different environments. Consider using `oci.manifest.digest` if it is important to identify the same image in different environments/runtimes.
///
/// # Examples
///
/// - `sha256:19c92d0a00d1b66d897bceaa7319bee0dd38a10a851c60bcec9474aa3f01e50f`
pub const CONTAINER_IMAGE_ID: &str = "container.image.id";
/// Name of the image the container was built on.
///
/// # Examples
///
/// - `gcr.io/opentelemetry/operator`
pub const CONTAINER_IMAGE_NAME: &str = "container.image.name";
/// Repo digests of the container image as provided by the container runtime.
///
/// [Docker](https://docs.docker.com/engine/api/v1.43/#tag/Image/operation/ImageInspect) and [CRI](https://github.com/kubernetes/cri-api/blob/c75ef5b473bbe2d0a4fc92f82235efd665ea8e9f/pkg/apis/runtime/v1/api.proto#L1237-L1238) report those under the `RepoDigests` field.
///
/// # Examples
///
/// - `example@sha256:afcc7f1ac1b49db317a7196c902e61c6c3c4607d63599ee1a82d702d249a0ccb`
/// - `internal.registry.example.com:5000/example@sha256:b69959407d21e8a062e0416bf13405bb2b71ed7a84dde4158ebafacfa06f5578`
pub const CONTAINER_IMAGE_REPO_DIGESTS: &str = "container.image.repo_digests";
/// Container image tags. An example can be found in [Docker Image Inspect](https://docs.docker.com/engine/api/v1.43/#tag/Image/operation/ImageInspect). Should be only the `&lt;tag&gt;` section of the full name for example from `registry.example.com/my-org/my-image:&lt;tag&gt;`.
///
/// # Examples
///
/// - `v1.27.1`
/// - `3.5.7-0`
pub const CONTAINER_IMAGE_TAGS: &str = "container.image.tags";
/// Container name used by container runtime.
///
/// # Examples
///
/// - `opentelemetry-autoconf`
pub const CONTAINER_NAME: &str = "container.name";
/// The container runtime managing this container.
///
/// # Examples
///
/// - `docker`
/// - `containerd`
/// - `rkt`
pub const CONTAINER_RUNTIME: &str = "container.runtime";
/// The consistency level of the query. Based on consistency values from [CQL](https://docs.datastax.com/en/cassandra-oss/3.0/cassandra/dml/dmlConfigConsistency.html).
pub const DB_CASSANDRA_CONSISTENCY_LEVEL: &str = "db.cassandra.consistency_level";
/// The data center of the coordinating node for a query.
///
/// # Examples
///
/// - `us-west-2`
pub const DB_CASSANDRA_COORDINATOR_DC: &str = "db.cassandra.coordinator.dc";
/// The ID of the coordinating node for a query.
///
/// # Examples
///
/// - `be13faa2-8574-4d71-926d-27f16cf8a7af`
pub const DB_CASSANDRA_COORDINATOR_ID: &str = "db.cassandra.coordinator.id";
/// Whether or not the query is idempotent.
pub const DB_CASSANDRA_IDEMPOTENCE: &str = "db.cassandra.idempotence";
/// The fetch size used for paging, i.e. how many rows will be returned at once.
///
/// # Examples
///
/// - `5000`
pub const DB_CASSANDRA_PAGE_SIZE: &str = "db.cassandra.page_size";
/// The number of times a query was speculatively executed. Not set or `0` if the query was not executed speculatively.
///
/// # Examples
///
/// - `0`
/// - `2`
pub const DB_CASSANDRA_SPECULATIVE_EXECUTION_COUNT: &str =
    "db.cassandra.speculative_execution_count";
/// Deprecated, use `db.collection.name` instead.
///
/// # Examples
///
/// - `mytable`
#[deprecated]
pub const DB_CASSANDRA_TABLE: &str = "db.cassandra.table";
/// The name of the connection pool; unique within the instrumented application. In case the connection pool implementation doesn&#39;t provide a name, instrumentation should use a combination of `server.address` and `server.port` attributes formatted as `server.address:server.port`.
///
/// # Examples
///
/// - `myDataSource`
pub const DB_CLIENT_CONNECTIONS_POOL_NAME: &str = "db.client.connections.pool.name";
/// The state of a connection in the pool.
///
/// # Examples
///
/// - `idle`
pub const DB_CLIENT_CONNECTIONS_STATE: &str = "db.client.connections.state";
/// The name of a collection (table, container) within the database.
///
/// If the collection name is parsed from the query, it SHOULD match the value provided in the query and may be qualified with the schema and database name.
/// It is RECOMMENDED to capture the value as provided by the application without attempting to do any case normalization.
///
/// # Examples
///
/// - `public.users`
/// - `customers`
pub const DB_COLLECTION_NAME: &str = "db.collection.name";
/// Deprecated, use `server.address`, `server.port` attributes instead.
///
/// # Examples
///
/// - `Server=(localdb)\v11.0;Integrated Security=true;`
#[deprecated]
pub const DB_CONNECTION_STRING: &str = "db.connection_string";
/// Unique Cosmos client instance id.
///
/// # Examples
///
/// - `3ba4827d-4422-483f-b59f-85b74211c11d`
pub const DB_COSMOSDB_CLIENT_ID: &str = "db.cosmosdb.client_id";
/// Cosmos client connection mode.
pub const DB_COSMOSDB_CONNECTION_MODE: &str = "db.cosmosdb.connection_mode";
/// Deprecated, use `db.collection.name` instead.
///
/// # Examples
///
/// - `mytable`
#[deprecated]
pub const DB_COSMOSDB_CONTAINER: &str = "db.cosmosdb.container";
/// CosmosDB Operation Type.
pub const DB_COSMOSDB_OPERATION_TYPE: &str = "db.cosmosdb.operation_type";
/// RU consumed for that operation.
///
/// # Examples
///
/// - `46.18`
/// - `1.0`
pub const DB_COSMOSDB_REQUEST_CHARGE: &str = "db.cosmosdb.request_charge";
/// Request payload size in bytes.
pub const DB_COSMOSDB_REQUEST_CONTENT_LENGTH: &str = "db.cosmosdb.request_content_length";
/// Cosmos DB status code.
///
/// # Examples
///
/// - `200`
/// - `201`
pub const DB_COSMOSDB_STATUS_CODE: &str = "db.cosmosdb.status_code";
/// Cosmos DB sub status code.
///
/// # Examples
///
/// - `1000`
/// - `1002`
pub const DB_COSMOSDB_SUB_STATUS_CODE: &str = "db.cosmosdb.sub_status_code";
/// Represents the identifier of an Elasticsearch cluster.
///
/// # Examples
///
/// - `e9106fc68e3044f0b1475b04bf4ffd5f`
pub const DB_ELASTICSEARCH_CLUSTER_NAME: &str = "db.elasticsearch.cluster.name";
/// Represents the human-readable identifier of the node/instance to which a request was routed.
///
/// # Examples
///
/// - `instance-0000000001`
pub const DB_ELASTICSEARCH_NODE_NAME: &str = "db.elasticsearch.node.name";
/// Deprecated, no general replacement at this time. For Elasticsearch, use `db.elasticsearch.node.name` instead.
///
/// # Examples
///
/// - `mysql-e26b99z.example.com`
#[deprecated]
pub const DB_INSTANCE_ID: &str = "db.instance.id";
/// Removed, no replacement at this time.
///
/// # Examples
///
/// - `org.postgresql.Driver`
/// - `com.microsoft.sqlserver.jdbc.SQLServerDriver`
#[deprecated]
pub const DB_JDBC_DRIVER_CLASSNAME: &str = "db.jdbc.driver_classname";
/// Deprecated, use `db.collection.name` instead.
///
/// # Examples
///
/// - `mytable`
#[deprecated]
pub const DB_MONGODB_COLLECTION: &str = "db.mongodb.collection";
/// Deprecated, SQL Server instance is now populated as a part of `db.namespace` attribute.
///
/// # Examples
///
/// - `MSSQLSERVER`
#[deprecated]
pub const DB_MSSQL_INSTANCE_NAME: &str = "db.mssql.instance_name";
/// Deprecated, use `db.namespace` instead.
///
/// # Examples
///
/// - `customers`
/// - `main`
#[deprecated]
pub const DB_NAME: &str = "db.name";
/// The name of the database, fully qualified within the server address and port.
///
/// If a database system has multiple namespace components, they SHOULD be concatenated (potentially using database system specific conventions) from most general to most specific namespace component, and more specific namespaces SHOULD NOT be captured without the more general namespaces, to ensure that &#34;startswith&#34; queries for the more general namespaces will be valid.
/// Semantic conventions for individual database systems SHOULD document what `db.namespace` means in the context of that system.
/// It is RECOMMENDED to capture the value as provided by the application without attempting to do any case normalization.
///
/// # Examples
///
/// - `customers`
/// - `test.users`
pub const DB_NAMESPACE: &str = "db.namespace";
/// Deprecated, use `db.operation.name` instead.
///
/// # Examples
///
/// - `findAndModify`
/// - `HMSET`
/// - `SELECT`
#[deprecated]
pub const DB_OPERATION: &str = "db.operation";
/// The name of the operation or command being executed.
///
/// It is RECOMMENDED to capture the value as provided by the application without attempting to do any case normalization.
///
/// # Examples
///
/// - `findAndModify`
/// - `HMSET`
/// - `SELECT`
pub const DB_OPERATION_NAME: &str = "db.operation.name";
/// The database query being executed.
///
/// # Examples
///
/// - `SELECT * FROM wuser_table where username = ?`
/// - `SET mykey "WuValue"`
pub const DB_QUERY_TEXT: &str = "db.query.text";
/// Deprecated, use `db.namespace` instead.
///
/// # Examples
///
/// - `0`
/// - `1`
/// - `15`
#[deprecated]
pub const DB_REDIS_DATABASE_INDEX: &str = "db.redis.database_index";
/// Deprecated, use `db.collection.name` instead.
///
/// # Examples
///
/// - `mytable`
#[deprecated]
pub const DB_SQL_TABLE: &str = "db.sql.table";
/// The database statement being executed.
///
/// # Examples
///
/// - `SELECT * FROM wuser_table`
/// - `SET mykey "WuValue"`
#[deprecated]
pub const DB_STATEMENT: &str = "db.statement";
/// The database management system (DBMS) product as identified by the client instrumentation.
///
/// The actual DBMS may differ from the one identified by the client. For example, when using PostgreSQL client libraries to connect to a CockroachDB, the `db.system` is set to `postgresql` based on the instrumentation&#39;s best knowledge.
pub const DB_SYSTEM: &str = "db.system";
/// Deprecated, no replacement at this time.
///
/// # Examples
///
/// - `readonly_user`
/// - `reporting_user`
#[deprecated]
pub const DB_USER: &str = "db.user";
/// Name of the [deployment environment](https://wikipedia.org/wiki/Deployment_environment) (aka deployment tier).
///
/// `deployment.environment` does not affect the uniqueness constraints defined through
/// the `service.namespace`, `service.name` and `service.instance.id` resource attributes.
/// This implies that resources carrying the following attribute combinations MUST be
/// considered to be identifying the same service:
///
/// * `service.name=frontend`, `deployment.environment=production`
/// * `service.name=frontend`, `deployment.environment=staging`.
///
/// # Examples
///
/// - `staging`
/// - `production`
pub const DEPLOYMENT_ENVIRONMENT: &str = "deployment.environment";
/// Destination address - domain name if available without reverse DNS lookup; otherwise, IP address or Unix domain socket name.
///
/// When observed from the source side, and when communicating through an intermediary, `destination.address` SHOULD represent the destination address behind any intermediaries, for example proxies, if it&#39;s available.
///
/// # Examples
///
/// - `destination.example.com`
/// - `10.1.2.80`
/// - `/tmp/my.sock`
pub const DESTINATION_ADDRESS: &str = "destination.address";
/// Destination port number.
///
/// # Examples
///
/// - `3389`
/// - `2888`
pub const DESTINATION_PORT: &str = "destination.port";
/// A unique identifier representing the device.
///
/// The device identifier MUST only be defined using the values outlined below. This value is not an advertising identifier and MUST NOT be used as such. On iOS (Swift or Objective-C), this value MUST be equal to the [vendor identifier](https://developer.apple.com/documentation/uikit/uidevice/1620059-identifierforvendor). On Android (Java or Kotlin), this value MUST be equal to the Firebase Installation ID or a globally unique UUID which is persisted across sessions in your application. More information can be found [here](https://developer.android.com/training/articles/user-data-ids) on best practices and exact implementation details. Caution should be taken when storing personal data or anything which can identify a user. GDPR and data protection laws may apply, ensure you do your own due diligence.
///
/// # Examples
///
/// - `2ab2916d-a51f-4ac8-80ee-45ac31a28092`
pub const DEVICE_ID: &str = "device.id";
/// The name of the device manufacturer.
///
/// The Android OS provides this field via [Build](https://developer.android.com/reference/android/os/Build#MANUFACTURER). iOS apps SHOULD hardcode the value `Apple`.
///
/// # Examples
///
/// - `Apple`
/// - `Samsung`
pub const DEVICE_MANUFACTURER: &str = "device.manufacturer";
/// The model identifier for the device.
///
/// It&#39;s recommended this value represents a machine-readable version of the model identifier rather than the market or consumer-friendly name of the device.
///
/// # Examples
///
/// - `iPhone3,4`
/// - `SM-G920F`
pub const DEVICE_MODEL_IDENTIFIER: &str = "device.model.identifier";
/// The marketing name for the device model.
///
/// It&#39;s recommended this value represents a human-readable version of the device model rather than a machine-readable alternative.
///
/// # Examples
///
/// - `iPhone 6s Plus`
/// - `Samsung Galaxy S6`
pub const DEVICE_MODEL_NAME: &str = "device.model.name";
/// The disk IO operation direction.
///
/// # Examples
///
/// - `read`
pub const DISK_IO_DIRECTION: &str = "disk.io.direction";
/// The name being queried.
///
/// If the name field contains non-printable characters (below 32 or above 126), those characters should be represented as escaped base 10 integers (\DDD). Back slashes and quotes should be escaped. Tabs, carriage returns, and line feeds should be converted to \t, \r, and \n respectively.
///
/// # Examples
///
/// - `www.example.com`
/// - `dot.net`
pub const DNS_QUESTION_NAME: &str = "dns.question.name";
/// Username or client_id extracted from the access token or [Authorization](https://tools.ietf.org/html/rfc7235#section-4.2) header in the inbound request from outside the system.
///
/// # Examples
///
/// - `username`
pub const ENDUSER_ID: &str = "enduser.id";
/// Actual/assumed role the client is making the request under extracted from token or application security context.
///
/// # Examples
///
/// - `admin`
pub const ENDUSER_ROLE: &str = "enduser.role";
/// Scopes or granted authorities the client currently possesses extracted from token or application security context. The value would come from the scope associated with an [OAuth 2.0 Access Token](https://tools.ietf.org/html/rfc6749#section-3.3) or an attribute value in a [SAML 2.0 Assertion](http://docs.oasis-open.org/security/saml/Post2.0/sstc-saml-tech-overview-2.0.html).
///
/// # Examples
///
/// - `read:message, write:files`
pub const ENDUSER_SCOPE: &str = "enduser.scope";
/// Describes a class of error the operation ended with.
///
/// The `error.type` SHOULD match the error code returned by the database or the client library, the canonical name of exception that occurred, or another low-cardinality error identifier. Instrumentations SHOULD document the list of errors they report.
///
/// # Examples
///
/// - `timeout`
/// - `java.net.UnknownHostException`
/// - `server_certificate_invalid`
/// - `500`
pub const ERROR_TYPE: &str = "error.type";
/// Identifies the class / type of event.
///
/// Event names are subject to the same rules as [attribute names](https://github.com/open-telemetry/opentelemetry-specification/tree/v1.33.0/specification/common/attribute-naming.md). Notably, event names are namespaced to avoid collisions and provide a clean separation of semantics for events in separate domains like browser, mobile, and kubernetes.
///
/// # Examples
///
/// - `browser.mouse.click`
/// - `device.app.lifecycle`
pub const EVENT_NAME: &str = "event.name";
/// SHOULD be set to true if the exception event is recorded at a point where it is known that the exception is escaping the scope of the span.
///
/// An exception is considered to have escaped (or left) the scope of a span,
/// if that span is ended while the exception is still logically &#34;in flight&#34;.
/// This may be actually &#34;in flight&#34; in some languages (e.g. if the exception
/// is passed to a Context manager&#39;s `__exit__` method in Python) but will
/// usually be caught at the point of recording the exception in most languages.
///
/// It is usually not possible to determine at the point where an exception is thrown
/// whether it will escape the scope of a span.
/// However, it is trivial to know that an exception
/// will escape, if one checks for an active exception just before ending the span,
/// as done in the [example for recording span exceptions](https://opentelemetry.io/docs/specs/semconv/exceptions/exceptions-spans/#recording-an-exception).
///
/// It follows that an exception may still escape the scope of the span
/// even if the `exception.escaped` attribute was not set or set to false,
/// since the event might have been recorded at a time where it was not
/// clear whether the exception will escape.
pub const EXCEPTION_ESCAPED: &str = "exception.escaped";
/// The exception message.
///
/// # Examples
///
/// - `Division by zero`
/// - `Can't convert 'int' object to str implicitly`
pub const EXCEPTION_MESSAGE: &str = "exception.message";
/// A stacktrace as a string in the natural representation for the language runtime. The representation is to be determined and documented by each language SIG.
///
/// # Examples
///
/// - `Exception in thread "main" java.lang.RuntimeException: Test exception\n at com.example.GenerateTrace.methodB(GenerateTrace.java:13)\n at com.example.GenerateTrace.methodA(GenerateTrace.java:9)\n at com.example.GenerateTrace.main(GenerateTrace.java:5)`
pub const EXCEPTION_STACKTRACE: &str = "exception.stacktrace";
/// The type of the exception (its fully-qualified class name, if applicable). The dynamic type of the exception should be preferred over the static type in languages that support it.
///
/// # Examples
///
/// - `java.net.ConnectException`
/// - `OSError`
pub const EXCEPTION_TYPE: &str = "exception.type";
/// A boolean that is true if the serverless function is executed for the first time (aka cold-start).
pub const FAAS_COLDSTART: &str = "faas.coldstart";
/// A string containing the schedule period as [Cron Expression](https://docs.oracle.com/cd/E12058_01/doc/doc.1014/e12030/cron_expressions.htm).
///
/// # Examples
///
/// - `0/5 * * * ? *`
pub const FAAS_CRON: &str = "faas.cron";
/// The name of the source on which the triggering operation was performed. For example, in Cloud Storage or S3 corresponds to the bucket name, and in Cosmos DB to the database name.
///
/// # Examples
///
/// - `myBucketName`
/// - `myDbName`
pub const FAAS_DOCUMENT_COLLECTION: &str = "faas.document.collection";
/// The document name/table subjected to the operation. For example, in Cloud Storage or S3 is the name of the file, and in Cosmos DB the table name.
///
/// # Examples
///
/// - `myFile.txt`
/// - `myTableName`
pub const FAAS_DOCUMENT_NAME: &str = "faas.document.name";
/// Describes the type of the operation that was performed on the data.
pub const FAAS_DOCUMENT_OPERATION: &str = "faas.document.operation";
/// A string containing the time when the data was accessed in the [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) format expressed in [UTC](https://www.w3.org/TR/NOTE-datetime).
///
/// # Examples
///
/// - `2020-01-23T13:47:06Z`
pub const FAAS_DOCUMENT_TIME: &str = "faas.document.time";
/// The execution environment ID as a string, that will be potentially reused for other invocations to the same function/function version.
///
/// * **AWS Lambda:** Use the (full) log stream name.
///
/// # Examples
///
/// - `2021/06/28/[$LATEST]2f399eb14537447da05ab2a2e39309de`
pub const FAAS_INSTANCE: &str = "faas.instance";
/// The invocation ID of the current function invocation.
///
/// # Examples
///
/// - `af9d5aa4-a685-4c5f-a22b-444f80b3cc28`
pub const FAAS_INVOCATION_ID: &str = "faas.invocation_id";
/// The name of the invoked function.
///
/// SHOULD be equal to the `faas.name` resource attribute of the invoked function.
///
/// # Examples
///
/// - `my-function`
pub const FAAS_INVOKED_NAME: &str = "faas.invoked_name";
/// The cloud provider of the invoked function.
///
/// SHOULD be equal to the `cloud.provider` resource attribute of the invoked function.
pub const FAAS_INVOKED_PROVIDER: &str = "faas.invoked_provider";
/// The cloud region of the invoked function.
///
/// SHOULD be equal to the `cloud.region` resource attribute of the invoked function.
///
/// # Examples
///
/// - `eu-central-1`
pub const FAAS_INVOKED_REGION: &str = "faas.invoked_region";
/// The amount of memory available to the serverless function converted to Bytes.
///
/// It&#39;s recommended to set this attribute since e.g. too little memory can easily stop a Java AWS Lambda function from working correctly. On AWS Lambda, the environment variable `AWS_LAMBDA_FUNCTION_MEMORY_SIZE` provides this information (which must be multiplied by 1,048,576).
///
/// # Examples
///
/// - `134217728`
pub const FAAS_MAX_MEMORY: &str = "faas.max_memory";
/// The name of the single function that this runtime instance executes.
///
/// This is the name of the function as configured/deployed on the FaaS
/// platform and is usually different from the name of the callback
/// function (which may be stored in the
/// [`code.namespace`/`code.function`](/docs/general/attributes.md#source-code-attributes)
/// span attributes).
///
/// For some cloud providers, the above definition is ambiguous. The following
/// definition of function name MUST be used for this attribute
/// (and consequently the span name) for the listed cloud providers/products:
///
/// * **Azure:**  The full name `&lt;FUNCAPP&gt;/&lt;FUNC&gt;`, i.e., function app name
///   followed by a forward slash followed by the function name (this form
///   can also be seen in the resource JSON for the function).
///   This means that a span attribute MUST be used, as an Azure function
///   app can host multiple functions that would usually share
///   a TracerProvider (see also the `cloud.resource_id` attribute).
///
/// # Examples
///
/// - `my-function`
/// - `myazurefunctionapp/some-function-name`
pub const FAAS_NAME: &str = "faas.name";
/// A string containing the function invocation time in the [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) format expressed in [UTC](https://www.w3.org/TR/NOTE-datetime).
///
/// # Examples
///
/// - `2020-01-23T13:47:06Z`
pub const FAAS_TIME: &str = "faas.time";
/// Type of the trigger which caused this function invocation.
pub const FAAS_TRIGGER: &str = "faas.trigger";
/// The immutable version of the function being executed.
///
/// Depending on the cloud provider and platform, use:
///
/// * **AWS Lambda:** The [function version](https://docs.aws.amazon.com/lambda/latest/dg/configuration-versions.html)
///   (an integer represented as a decimal string).
/// * **Google Cloud Run (Services):** The [revision](https://cloud.google.com/run/docs/managing/revisions)
///   (i.e., the function name plus the revision suffix).
/// * **Google Cloud Functions:** The value of the
///   [`K_REVISION` environment variable](https://cloud.google.com/functions/docs/env-var#runtime_environment_variables_set_automatically).
/// * **Azure Functions:** Not applicable. Do not set this attribute.
///
/// # Examples
///
/// - `26`
/// - `pinkfroid-00002`
pub const FAAS_VERSION: &str = "faas.version";
/// The unique identifier of the feature flag.
///
/// # Examples
///
/// - `logo-color`
pub const FEATURE_FLAG_KEY: &str = "feature_flag.key";
/// The name of the service provider that performs the flag evaluation.
///
/// # Examples
///
/// - `Flag Manager`
pub const FEATURE_FLAG_PROVIDER_NAME: &str = "feature_flag.provider_name";
/// SHOULD be a semantic identifier for a value. If one is unavailable, a stringified version of the value can be used.
///
/// A semantic identifier, commonly referred to as a variant, provides a means
/// for referring to a value without including the value itself. This can
/// provide additional context for understanding the meaning behind a value.
/// For example, the variant `red` maybe be used for the value `#c05543`.
///
/// A stringified version of the value can be used in situations where a
/// semantic identifier is unavailable. String representation of the value
/// should be determined by the implementer.
///
/// # Examples
///
/// - `red`
/// - `true`
/// - `on`
pub const FEATURE_FLAG_VARIANT: &str = "feature_flag.variant";
/// Directory where the file is located. It should include the drive letter, when appropriate.
///
/// # Examples
///
/// - `/home/user`
/// - `C:\Program Files\MyApp`
pub const FILE_DIRECTORY: &str = "file.directory";
/// File extension, excluding the leading dot.
///
/// When the file name has multiple extensions (example.tar.gz), only the last one should be captured (&#34;gz&#34;, not &#34;tar.gz&#34;).
///
/// # Examples
///
/// - `png`
/// - `gz`
pub const FILE_EXTENSION: &str = "file.extension";
/// Name of the file including the extension, without the directory.
///
/// # Examples
///
/// - `example.png`
pub const FILE_NAME: &str = "file.name";
/// Full path to the file, including the file name. It should include the drive letter, when appropriate.
///
/// # Examples
///
/// - `/home/alice/example.png`
/// - `C:\Program Files\MyApp\myapp.exe`
pub const FILE_PATH: &str = "file.path";
/// File size in bytes.
pub const FILE_SIZE: &str = "file.size";
/// The name of the Cloud Run [execution](https://cloud.google.com/run/docs/managing/job-executions) being run for the Job, as set by the [`CLOUD_RUN_EXECUTION`](https://cloud.google.com/run/docs/container-contract#jobs-env-vars) environment variable.
///
/// # Examples
///
/// - `job-name-xxxx`
/// - `sample-job-mdw84`
pub const GCP_CLOUD_RUN_JOB_EXECUTION: &str = "gcp.cloud_run.job.execution";
/// The index for a task within an execution as provided by the [`CLOUD_RUN_TASK_INDEX`](https://cloud.google.com/run/docs/container-contract#jobs-env-vars) environment variable.
///
/// # Examples
///
/// - `0`
/// - `1`
pub const GCP_CLOUD_RUN_JOB_TASK_INDEX: &str = "gcp.cloud_run.job.task_index";
/// The hostname of a GCE instance. This is the full value of the default or [custom hostname](https://cloud.google.com/compute/docs/instances/custom-hostname-vm).
///
/// # Examples
///
/// - `my-host1234.example.com`
/// - `sample-vm.us-west1-b.c.my-project.internal`
pub const GCP_GCE_INSTANCE_HOSTNAME: &str = "gcp.gce.instance.hostname";
/// The instance name of a GCE instance. This is the value provided by `host.name`, the visible name of the instance in the Cloud Console UI, and the prefix for the default hostname of the instance as defined by the [default internal DNS name](https://cloud.google.com/compute/docs/internal-dns#instance-fully-qualified-domain-names).
///
/// # Examples
///
/// - `instance-1`
/// - `my-vm-name`
pub const GCP_GCE_INSTANCE_NAME: &str = "gcp.gce.instance.name";
/// The full response received from the LLM.
///
/// It&#39;s RECOMMENDED to format completions as JSON string matching [OpenAI messages format](https://platform.openai.com/docs/guides/text-generation)
///
/// # Examples
///
/// - `[{'role': 'assistant', 'content': 'The capital of France is Paris.'}]`
pub const GEN_AI_COMPLETION: &str = "gen_ai.completion";
/// The full prompt sent to an LLM.
///
/// It&#39;s RECOMMENDED to format prompts as JSON string matching [OpenAI messages format](https://platform.openai.com/docs/guides/text-generation)
///
/// # Examples
///
/// - `[{'role': 'user', 'content': 'What is the capital of France?'}]`
pub const GEN_AI_PROMPT: &str = "gen_ai.prompt";
/// The maximum number of tokens the LLM generates for a request.
///
/// # Examples
///
/// - `100`
pub const GEN_AI_REQUEST_MAX_TOKENS: &str = "gen_ai.request.max_tokens";
/// The name of the LLM a request is being made to.
///
/// # Examples
///
/// - `gpt-4`
pub const GEN_AI_REQUEST_MODEL: &str = "gen_ai.request.model";
/// The temperature setting for the LLM request.
///
/// # Examples
///
/// - `0.0`
pub const GEN_AI_REQUEST_TEMPERATURE: &str = "gen_ai.request.temperature";
/// The top_p sampling setting for the LLM request.
///
/// # Examples
///
/// - `1.0`
pub const GEN_AI_REQUEST_TOP_P: &str = "gen_ai.request.top_p";
/// Array of reasons the model stopped generating tokens, corresponding to each generation received.
///
/// # Examples
///
/// - `stop`
pub const GEN_AI_RESPONSE_FINISH_REASONS: &str = "gen_ai.response.finish_reasons";
/// The unique identifier for the completion.
///
/// # Examples
///
/// - `chatcmpl-123`
pub const GEN_AI_RESPONSE_ID: &str = "gen_ai.response.id";
/// The name of the LLM a response was generated from.
///
/// # Examples
///
/// - `gpt-4-0613`
pub const GEN_AI_RESPONSE_MODEL: &str = "gen_ai.response.model";
/// The Generative AI product as identified by the client instrumentation.
///
/// The actual GenAI product may differ from the one identified by the client. For example, when using OpenAI client libraries to communicate with Mistral, the `gen_ai.system` is set to `openai` based on the instrumentation&#39;s best knowledge.
///
/// # Examples
///
/// - `openai`
pub const GEN_AI_SYSTEM: &str = "gen_ai.system";
/// The number of tokens used in the LLM response (completion).
///
/// # Examples
///
/// - `180`
pub const GEN_AI_USAGE_COMPLETION_TOKENS: &str = "gen_ai.usage.completion_tokens";
/// The number of tokens used in the LLM prompt.
///
/// # Examples
///
/// - `100`
pub const GEN_AI_USAGE_PROMPT_TOKENS: &str = "gen_ai.usage.prompt_tokens";
/// The GraphQL document being executed.
///
/// The value may be sanitized to exclude sensitive information.
///
/// # Examples
///
/// - `query findBookById { bookById(id: ?) { name } }`
pub const GRAPHQL_DOCUMENT: &str = "graphql.document";
/// The name of the operation being executed.
///
/// # Examples
///
/// - `findBookById`
pub const GRAPHQL_OPERATION_NAME: &str = "graphql.operation.name";
/// The type of the operation being executed.
///
/// # Examples
///
/// - `query`
/// - `mutation`
/// - `subscription`
pub const GRAPHQL_OPERATION_TYPE: &str = "graphql.operation.type";
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
/// The CPU architecture the host system is running on.
pub const HOST_ARCH: &str = "host.arch";
/// The amount of level 2 memory cache available to the processor (in Bytes).
///
/// # Examples
///
/// - `12288000`
pub const HOST_CPU_CACHE_L2_SIZE: &str = "host.cpu.cache.l2.size";
/// Family or generation of the CPU.
///
/// # Examples
///
/// - `6`
/// - `PA-RISC 1.1e`
pub const HOST_CPU_FAMILY: &str = "host.cpu.family";
/// Model identifier. It provides more granular information about the CPU, distinguishing it from other CPUs within the same family.
///
/// # Examples
///
/// - `6`
/// - `9000/778/B180L`
pub const HOST_CPU_MODEL_ID: &str = "host.cpu.model.id";
/// Model designation of the processor.
///
/// # Examples
///
/// - `11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz`
pub const HOST_CPU_MODEL_NAME: &str = "host.cpu.model.name";
/// Stepping or core revisions.
///
/// # Examples
///
/// - `1`
/// - `r1p1`
pub const HOST_CPU_STEPPING: &str = "host.cpu.stepping";
/// Processor manufacturer identifier. A maximum 12-character string.
///
/// [CPUID](https://wiki.osdev.org/CPUID) command returns the vendor ID string in EBX, EDX and ECX registers. Writing these to memory in this order results in a 12-character string.
///
/// # Examples
///
/// - `GenuineIntel`
pub const HOST_CPU_VENDOR_ID: &str = "host.cpu.vendor.id";
/// Unique host ID. For Cloud, this must be the instance_id assigned by the cloud provider. For non-containerized systems, this should be the `machine-id`. See the table below for the sources to use to determine the `machine-id` based on operating system.
///
/// # Examples
///
/// - `fdbf79e8af94cb7f9e8df36789187052`
pub const HOST_ID: &str = "host.id";
/// VM image ID or host OS image ID. For Cloud, this value is from the provider.
///
/// # Examples
///
/// - `ami-07b06b442921831e5`
pub const HOST_IMAGE_ID: &str = "host.image.id";
/// Name of the VM image or OS install the host was instantiated from.
///
/// # Examples
///
/// - `infra-ami-eks-worker-node-7d4ec78312`
/// - `CentOS-8-x86_64-1905`
pub const HOST_IMAGE_NAME: &str = "host.image.name";
/// The version string of the VM image or host OS as defined in [Version Attributes](/docs/resource/README.md#version-attributes).
///
/// # Examples
///
/// - `0.1`
pub const HOST_IMAGE_VERSION: &str = "host.image.version";
/// Available IP addresses of the host, excluding loopback interfaces.
///
/// IPv4 Addresses MUST be specified in dotted-quad notation. IPv6 addresses MUST be specified in the [RFC 5952](https://www.rfc-editor.org/rfc/rfc5952.html) format.
///
/// # Examples
///
/// - `192.168.1.140`
/// - `fe80::abc2:4a28:737a:609e`
pub const HOST_IP: &str = "host.ip";
/// Available MAC addresses of the host, excluding loopback interfaces.
///
/// MAC Addresses MUST be represented in [IEEE RA hexadecimal form](https://standards.ieee.org/wp-content/uploads/import/documents/tutorials/eui.pdf): as hyphen-separated octets in uppercase hexadecimal form from most to least significant.
///
/// # Examples
///
/// - `AC-DE-48-23-45-67`
/// - `AC-DE-48-23-45-67-01-9F`
pub const HOST_MAC: &str = "host.mac";
/// Name of the host. On Unix systems, it may contain what the hostname command returns, or the fully qualified hostname, or another name specified by the user.
///
/// # Examples
///
/// - `opentelemetry-test`
pub const HOST_NAME: &str = "host.name";
/// Type of host. For Cloud, this must be the machine type.
///
/// # Examples
///
/// - `n1-standard-1`
pub const HOST_TYPE: &str = "host.type";
/// Deprecated, use `client.address` instead.
///
/// # Examples
///
/// - `83.164.160.102`
#[deprecated]
pub const HTTP_CLIENT_IP: &str = "http.client_ip";
/// State of the HTTP connection in the HTTP connection pool.
///
/// # Examples
///
/// - `active`
/// - `idle`
pub const HTTP_CONNECTION_STATE: &str = "http.connection.state";
/// Deprecated, use `network.protocol.name` instead.
#[deprecated]
pub const HTTP_FLAVOR: &str = "http.flavor";
/// Deprecated, use one of `server.address`, `client.address` or `http.request.header.host` instead, depending on the usage.
///
/// # Examples
///
/// - `www.example.org`
#[deprecated]
pub const HTTP_HOST: &str = "http.host";
/// Deprecated, use `http.request.method` instead.
///
/// # Examples
///
/// - `GET`
/// - `POST`
/// - `HEAD`
#[deprecated]
pub const HTTP_METHOD: &str = "http.method";
/// The size of the request payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
///
/// # Examples
///
/// - `3495`
pub const HTTP_REQUEST_BODY_SIZE: &str = "http.request.body.size";
/// Deprecated, use `http.request.header.content-length` instead.
///
/// # Examples
///
/// - `3495`
#[deprecated]
pub const HTTP_REQUEST_CONTENT_LENGTH: &str = "http.request_content_length";
/// Deprecated, use `http.request.body.size` instead.
///
/// # Examples
///
/// - `5493`
#[deprecated]
pub const HTTP_REQUEST_CONTENT_LENGTH_UNCOMPRESSED: &str =
    "http.request_content_length_uncompressed";
/// HTTP request method.
///
/// HTTP request method value SHOULD be &#34;known&#34; to the instrumentation.
/// By default, this convention defines &#34;known&#34; methods as the ones listed in [RFC9110](https://www.rfc-editor.org/rfc/rfc9110.html#name-methods)
/// and the PATCH method defined in [RFC5789](https://www.rfc-editor.org/rfc/rfc5789.html).
///
/// If the HTTP request method is not known to instrumentation, it MUST set the `http.request.method` attribute to `_OTHER`.
///
/// If the HTTP instrumentation could end up converting valid HTTP request methods to `_OTHER`, then it MUST provide a way to override
/// the list of known HTTP methods. If this override is done via environment variable, then the environment variable MUST be named
/// OTEL_INSTRUMENTATION_HTTP_KNOWN_METHODS and support a comma-separated list of case-sensitive known HTTP methods
/// (this list MUST be a full override of the default known method, it is not a list of known methods in addition to the defaults).
///
/// HTTP method names are case-sensitive and `http.request.method` attribute value MUST match a known HTTP method name exactly.
/// Instrumentations for specific web frameworks that consider HTTP methods to be case insensitive, SHOULD populate a canonical equivalent.
/// Tracing instrumentations that do so, MUST also set `http.request.method_original` to the original value.
///
/// # Examples
///
/// - `GET`
/// - `POST`
/// - `HEAD`
pub const HTTP_REQUEST_METHOD: &str = "http.request.method";
/// Original HTTP method sent by the client in the request line.
///
/// # Examples
///
/// - `GeT`
/// - `ACL`
/// - `foo`
pub const HTTP_REQUEST_METHOD_ORIGINAL: &str = "http.request.method_original";
/// The ordinal number of request resending attempt (for any reason, including redirects).
///
/// The resend count SHOULD be updated each time an HTTP request gets resent by the client, regardless of what was the cause of the resending (e.g. redirection, authorization failure, 503 Server Unavailable, network issues, or any other).
///
/// # Examples
///
/// - `3`
pub const HTTP_REQUEST_RESEND_COUNT: &str = "http.request.resend_count";
/// The total size of the request in bytes. This should be the total number of bytes sent over the wire, including the request line (HTTP/1.1), framing (HTTP/2 and HTTP/3), headers, and request body if any.
///
/// # Examples
///
/// - `1437`
pub const HTTP_REQUEST_SIZE: &str = "http.request.size";
/// The size of the response payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
///
/// # Examples
///
/// - `3495`
pub const HTTP_RESPONSE_BODY_SIZE: &str = "http.response.body.size";
/// Deprecated, use `http.response.header.content-length` instead.
///
/// # Examples
///
/// - `3495`
#[deprecated]
pub const HTTP_RESPONSE_CONTENT_LENGTH: &str = "http.response_content_length";
/// Deprecated, use `http.response.body.size` instead.
///
/// # Examples
///
/// - `5493`
#[deprecated]
pub const HTTP_RESPONSE_CONTENT_LENGTH_UNCOMPRESSED: &str =
    "http.response_content_length_uncompressed";
/// The total size of the response in bytes. This should be the total number of bytes sent over the wire, including the status line (HTTP/1.1), framing (HTTP/2 and HTTP/3), headers, and response body and trailers if any.
///
/// # Examples
///
/// - `1437`
pub const HTTP_RESPONSE_SIZE: &str = "http.response.size";
/// [HTTP response status code](https://tools.ietf.org/html/rfc7231#section-6).
///
/// # Examples
///
/// - `200`
pub const HTTP_RESPONSE_STATUS_CODE: &str = "http.response.status_code";
/// The matched route, that is, the path template in the format used by the respective server framework.
///
/// MUST NOT be populated when this is not supported by the HTTP server framework as the route attribute should have low-cardinality and the URI path can NOT substitute it.
/// SHOULD include the [application root](/docs/http/http-spans.md#http-server-definitions) if there is one.
///
/// # Examples
///
/// - `/users/:userID?`
/// - `{controller}/{action}/{id?}`
pub const HTTP_ROUTE: &str = "http.route";
/// Deprecated, use `url.scheme` instead.
///
/// # Examples
///
/// - `http`
/// - `https`
#[deprecated]
pub const HTTP_SCHEME: &str = "http.scheme";
/// Deprecated, use `server.address` instead.
///
/// # Examples
///
/// - `example.com`
#[deprecated]
pub const HTTP_SERVER_NAME: &str = "http.server_name";
/// Deprecated, use `http.response.status_code` instead.
///
/// # Examples
///
/// - `200`
#[deprecated]
pub const HTTP_STATUS_CODE: &str = "http.status_code";
/// Deprecated, use `url.path` and `url.query` instead.
///
/// # Examples
///
/// - `/search?q=OpenTelemetry#SemConv`
#[deprecated]
pub const HTTP_TARGET: &str = "http.target";
/// Deprecated, use `url.full` instead.
///
/// # Examples
///
/// - `https://www.foo.bar/search?q=OpenTelemetry#SemConv`
#[deprecated]
pub const HTTP_URL: &str = "http.url";
/// Deprecated, use `user_agent.original` instead.
///
/// # Examples
///
/// - `CERN-LineMode/2.15 libwww/2.17b3`
/// - `Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.2 Mobile/15E148 Safari/604.1`
#[deprecated]
pub const HTTP_USER_AGENT: &str = "http.user_agent";
/// Deprecated use the `device.app.lifecycle` event definition including `ios.state` as a payload field instead.
///
/// The iOS lifecycle states are defined in the [UIApplicationDelegate documentation](https://developer.apple.com/documentation/uikit/uiapplicationdelegate#1656902), and from which the `OS terminology` column values are derived.
#[deprecated]
pub const IOS_STATE: &str = "ios.state";
/// Name of the buffer pool.
///
/// Pool names are generally obtained via [BufferPoolMXBean#getName()](https://docs.oracle.com/en/java/javase/11/docs/api/java.management/java/lang/management/BufferPoolMXBean.html#getName()).
///
/// # Examples
///
/// - `mapped`
/// - `direct`
pub const JVM_BUFFER_POOL_NAME: &str = "jvm.buffer.pool.name";
/// Name of the garbage collector action.
///
/// Garbage collector action is generally obtained via [GarbageCollectionNotificationInfo#getGcAction()](https://docs.oracle.com/en/java/javase/11/docs/api/jdk.management/com/sun/management/GarbageCollectionNotificationInfo.html#getGcAction()).
///
/// # Examples
///
/// - `end of minor GC`
/// - `end of major GC`
pub const JVM_GC_ACTION: &str = "jvm.gc.action";
/// Name of the garbage collector.
///
/// Garbage collector name is generally obtained via [GarbageCollectionNotificationInfo#getGcName()](https://docs.oracle.com/en/java/javase/11/docs/api/jdk.management/com/sun/management/GarbageCollectionNotificationInfo.html#getGcName()).
///
/// # Examples
///
/// - `G1 Young Generation`
/// - `G1 Old Generation`
pub const JVM_GC_NAME: &str = "jvm.gc.name";
/// Name of the memory pool.
///
/// Pool names are generally obtained via [MemoryPoolMXBean#getName()](https://docs.oracle.com/en/java/javase/11/docs/api/java.management/java/lang/management/MemoryPoolMXBean.html#getName()).
///
/// # Examples
///
/// - `G1 Old Gen`
/// - `G1 Eden space`
/// - `G1 Survivor Space`
pub const JVM_MEMORY_POOL_NAME: &str = "jvm.memory.pool.name";
/// The type of memory.
///
/// # Examples
///
/// - `heap`
/// - `non_heap`
pub const JVM_MEMORY_TYPE: &str = "jvm.memory.type";
/// Whether the thread is daemon or not.
pub const JVM_THREAD_DAEMON: &str = "jvm.thread.daemon";
/// State of the thread.
///
/// # Examples
///
/// - `runnable`
/// - `blocked`
pub const JVM_THREAD_STATE: &str = "jvm.thread.state";
/// The name of the cluster.
///
/// # Examples
///
/// - `opentelemetry-cluster`
pub const K8S_CLUSTER_NAME: &str = "k8s.cluster.name";
/// A pseudo-ID for the cluster, set to the UID of the `kube-system` namespace.
///
/// K8s doesn&#39;t have support for obtaining a cluster ID. If this is ever
/// added, we will recommend collecting the `k8s.cluster.uid` through the
/// official APIs. In the meantime, we are able to use the `uid` of the
/// `kube-system` namespace as a proxy for cluster ID. Read on for the
/// rationale.
///
/// Every object created in a K8s cluster is assigned a distinct UID. The
/// `kube-system` namespace is used by Kubernetes itself and will exist
/// for the lifetime of the cluster. Using the `uid` of the `kube-system`
/// namespace is a reasonable proxy for the K8s ClusterID as it will only
/// change if the cluster is rebuilt. Furthermore, Kubernetes UIDs are
/// UUIDs as standardized by
/// [ISO/IEC 9834-8 and ITU-T X.667](https://www.itu.int/ITU-T/studygroups/com17/oid.html).
/// Which states:
///
/// &gt; If generated according to one of the mechanisms defined in Rec.
///   ITU-T X.667 | ISO/IEC 9834-8, a UUID is either guaranteed to be
///   different from all other UUIDs generated before 3603 A.D., or is
///   extremely likely to be different (depending on the mechanism chosen).
///
/// Therefore, UIDs between clusters should be extremely unlikely to
/// conflict.
///
/// # Examples
///
/// - `218fc5a9-a5f1-4b54-aa05-46717d0ab26d`
pub const K8S_CLUSTER_UID: &str = "k8s.cluster.uid";
/// The name of the Container from Pod specification, must be unique within a Pod. Container runtime usually uses different globally unique name (`container.name`).
///
/// # Examples
///
/// - `redis`
pub const K8S_CONTAINER_NAME: &str = "k8s.container.name";
/// Number of times the container was restarted. This attribute can be used to identify a particular container (running or stopped) within a container spec.
pub const K8S_CONTAINER_RESTART_COUNT: &str = "k8s.container.restart_count";
/// Last terminated reason of the Container.
///
/// # Examples
///
/// - `Evicted`
/// - `Error`
pub const K8S_CONTAINER_STATUS_LAST_TERMINATED_REASON: &str =
    "k8s.container.status.last_terminated_reason";
/// The name of the CronJob.
///
/// # Examples
///
/// - `opentelemetry`
pub const K8S_CRONJOB_NAME: &str = "k8s.cronjob.name";
/// The UID of the CronJob.
///
/// # Examples
///
/// - `275ecb36-5aa8-4c2a-9c47-d8bb681b9aff`
pub const K8S_CRONJOB_UID: &str = "k8s.cronjob.uid";
/// The name of the DaemonSet.
///
/// # Examples
///
/// - `opentelemetry`
pub const K8S_DAEMONSET_NAME: &str = "k8s.daemonset.name";
/// The UID of the DaemonSet.
///
/// # Examples
///
/// - `275ecb36-5aa8-4c2a-9c47-d8bb681b9aff`
pub const K8S_DAEMONSET_UID: &str = "k8s.daemonset.uid";
/// The name of the Deployment.
///
/// # Examples
///
/// - `opentelemetry`
pub const K8S_DEPLOYMENT_NAME: &str = "k8s.deployment.name";
/// The UID of the Deployment.
///
/// # Examples
///
/// - `275ecb36-5aa8-4c2a-9c47-d8bb681b9aff`
pub const K8S_DEPLOYMENT_UID: &str = "k8s.deployment.uid";
/// The name of the Job.
///
/// # Examples
///
/// - `opentelemetry`
pub const K8S_JOB_NAME: &str = "k8s.job.name";
/// The UID of the Job.
///
/// # Examples
///
/// - `275ecb36-5aa8-4c2a-9c47-d8bb681b9aff`
pub const K8S_JOB_UID: &str = "k8s.job.uid";
/// The name of the namespace that the pod is running in.
///
/// # Examples
///
/// - `default`
pub const K8S_NAMESPACE_NAME: &str = "k8s.namespace.name";
/// The name of the Node.
///
/// # Examples
///
/// - `node-1`
pub const K8S_NODE_NAME: &str = "k8s.node.name";
/// The UID of the Node.
///
/// # Examples
///
/// - `1eb3a0c6-0477-4080-a9cb-0cb7db65c6a2`
pub const K8S_NODE_UID: &str = "k8s.node.uid";
/// The name of the Pod.
///
/// # Examples
///
/// - `opentelemetry-pod-autoconf`
pub const K8S_POD_NAME: &str = "k8s.pod.name";
/// The UID of the Pod.
///
/// # Examples
///
/// - `275ecb36-5aa8-4c2a-9c47-d8bb681b9aff`
pub const K8S_POD_UID: &str = "k8s.pod.uid";
/// The name of the ReplicaSet.
///
/// # Examples
///
/// - `opentelemetry`
pub const K8S_REPLICASET_NAME: &str = "k8s.replicaset.name";
/// The UID of the ReplicaSet.
///
/// # Examples
///
/// - `275ecb36-5aa8-4c2a-9c47-d8bb681b9aff`
pub const K8S_REPLICASET_UID: &str = "k8s.replicaset.uid";
/// The name of the StatefulSet.
///
/// # Examples
///
/// - `opentelemetry`
pub const K8S_STATEFULSET_NAME: &str = "k8s.statefulset.name";
/// The UID of the StatefulSet.
///
/// # Examples
///
/// - `275ecb36-5aa8-4c2a-9c47-d8bb681b9aff`
pub const K8S_STATEFULSET_UID: &str = "k8s.statefulset.uid";
/// The basename of the file.
///
/// # Examples
///
/// - `audit.log`
pub const LOG_FILE_NAME: &str = "log.file.name";
/// The basename of the file, with symlinks resolved.
///
/// # Examples
///
/// - `uuid.log`
pub const LOG_FILE_NAME_RESOLVED: &str = "log.file.name_resolved";
/// The full path to the file.
///
/// # Examples
///
/// - `/var/log/mysql/audit.log`
pub const LOG_FILE_PATH: &str = "log.file.path";
/// The full path to the file, with symlinks resolved.
///
/// # Examples
///
/// - `/var/lib/docker/uuid.log`
pub const LOG_FILE_PATH_RESOLVED: &str = "log.file.path_resolved";
/// The stream associated with the log. See below for a list of well-known values.
pub const LOG_IOSTREAM: &str = "log.iostream";
/// A unique identifier for the Log Record.
///
/// If an id is provided, other log records with the same id will be considered duplicates and can be removed safely. This means, that two distinguishable log records MUST have different values.
/// The id MAY be an [Universally Unique Lexicographically Sortable Identifier (ULID)](https://github.com/ulid/spec), but other identifiers (e.g. UUID) may be used as needed.
///
/// # Examples
///
/// - `01ARZ3NDEKTSV4RRFFQ69G5FAV`
pub const LOG_RECORD_UID: &str = "log.record.uid";
/// Deprecated, use `rpc.message.compressed_size` instead.
#[deprecated]
pub const MESSAGE_COMPRESSED_SIZE: &str = "message.compressed_size";
/// Deprecated, use `rpc.message.id` instead.
#[deprecated]
pub const MESSAGE_ID: &str = "message.id";
/// Deprecated, use `rpc.message.type` instead.
#[deprecated]
pub const MESSAGE_TYPE: &str = "message.type";
/// Deprecated, use `rpc.message.uncompressed_size` instead.
#[deprecated]
pub const MESSAGE_UNCOMPRESSED_SIZE: &str = "message.uncompressed_size";
/// The number of messages sent, received, or processed in the scope of the batching operation.
///
/// Instrumentations SHOULD NOT set `messaging.batch.message_count` on spans that operate with a single message. When a messaging client library supports both batch and single-message API for the same operation, instrumentations SHOULD use `messaging.batch.message_count` for batching APIs and SHOULD NOT use it for single-message APIs.
///
/// # Examples
///
/// - `0`
/// - `1`
/// - `2`
pub const MESSAGING_BATCH_MESSAGE_COUNT: &str = "messaging.batch.message_count";
/// A unique identifier for the client that consumes or produces a message.
///
/// # Examples
///
/// - `client-5`
/// - `myhost@8742@s8083jm`
pub const MESSAGING_CLIENT_ID: &str = "messaging.client.id";
/// A boolean that is true if the message destination is anonymous (could be unnamed or have auto-generated name).
pub const MESSAGING_DESTINATION_ANONYMOUS: &str = "messaging.destination.anonymous";
/// The message destination name.
///
/// Destination name SHOULD uniquely identify a specific queue, topic or other entity within the broker. If
/// the broker doesn&#39;t have such notion, the destination name SHOULD uniquely identify the broker.
///
/// # Examples
///
/// - `MyQueue`
/// - `MyTopic`
pub const MESSAGING_DESTINATION_NAME: &str = "messaging.destination.name";
/// The identifier of the partition messages are sent to or received from, unique within the `messaging.destination.name`.
///
/// # Examples
///
/// - `1`
pub const MESSAGING_DESTINATION_PARTITION_ID: &str = "messaging.destination.partition.id";
/// A boolean that is true if the publish message destination is anonymous (could be unnamed or have auto-generated name).
pub const MESSAGING_DESTINATION_PUBLISH_ANONYMOUS: &str = "messaging.destination_publish.anonymous";
/// The name of the original destination the message was published to.
///
/// The name SHOULD uniquely identify a specific queue, topic, or other entity within the broker. If
/// the broker doesn&#39;t have such notion, the original destination name SHOULD uniquely identify the broker.
///
/// # Examples
///
/// - `MyQueue`
/// - `MyTopic`
pub const MESSAGING_DESTINATION_PUBLISH_NAME: &str = "messaging.destination_publish.name";
/// Low cardinality representation of the messaging destination name.
///
/// Destination names could be constructed from templates. An example would be a destination name involving a user name or product id. Although the destination name in this case is of high cardinality, the underlying template is of low cardinality and can be effectively used for grouping and aggregation.
///
/// # Examples
///
/// - `/customers/{customerId}`
pub const MESSAGING_DESTINATION_TEMPLATE: &str = "messaging.destination.template";
/// A boolean that is true if the message destination is temporary and might not exist anymore after messages are processed.
pub const MESSAGING_DESTINATION_TEMPORARY: &str = "messaging.destination.temporary";
/// The name of the consumer group the event consumer is associated with.
///
/// # Examples
///
/// - `indexer`
pub const MESSAGING_EVENTHUBS_CONSUMER_GROUP: &str = "messaging.eventhubs.consumer.group";
/// The UTC epoch seconds at which the message has been accepted and stored in the entity.
///
/// # Examples
///
/// - `1701393730`
pub const MESSAGING_EVENTHUBS_MESSAGE_ENQUEUED_TIME: &str =
    "messaging.eventhubs.message.enqueued_time";
/// The ack deadline in seconds set for the modify ack deadline request.
///
/// # Examples
///
/// - `10`
pub const MESSAGING_GCP_PUBSUB_MESSAGE_ACK_DEADLINE: &str =
    "messaging.gcp_pubsub.message.ack_deadline";
/// The ack id for a given message.
///
/// # Examples
///
/// - `ack_id`
pub const MESSAGING_GCP_PUBSUB_MESSAGE_ACK_ID: &str = "messaging.gcp_pubsub.message.ack_id";
/// The delivery attempt for a given message.
///
/// # Examples
///
/// - `2`
pub const MESSAGING_GCP_PUBSUB_MESSAGE_DELIVERY_ATTEMPT: &str =
    "messaging.gcp_pubsub.message.delivery_attempt";
/// The ordering key for a given message. If the attribute is not present, the message does not have an ordering key.
///
/// # Examples
///
/// - `ordering_key`
pub const MESSAGING_GCP_PUBSUB_MESSAGE_ORDERING_KEY: &str =
    "messaging.gcp_pubsub.message.ordering_key";
/// Name of the Kafka Consumer Group that is handling the message. Only applies to consumers, not producers.
///
/// # Examples
///
/// - `my-group`
pub const MESSAGING_KAFKA_CONSUMER_GROUP: &str = "messaging.kafka.consumer.group";
/// Deprecated, use `messaging.destination.partition.id` instead.
///
/// # Examples
///
/// - `2`
#[deprecated]
pub const MESSAGING_KAFKA_DESTINATION_PARTITION: &str = "messaging.kafka.destination.partition";
/// Message keys in Kafka are used for grouping alike messages to ensure they&#39;re processed on the same partition. They differ from `messaging.message.id` in that they&#39;re not unique. If the key is `null`, the attribute MUST NOT be set.
///
/// If the key type is not string, it&#39;s string representation has to be supplied for the attribute. If the key has no unambiguous, canonical string form, don&#39;t include its value.
///
/// # Examples
///
/// - `myKey`
pub const MESSAGING_KAFKA_MESSAGE_KEY: &str = "messaging.kafka.message.key";
/// The offset of a record in the corresponding Kafka partition.
///
/// # Examples
///
/// - `42`
pub const MESSAGING_KAFKA_MESSAGE_OFFSET: &str = "messaging.kafka.message.offset";
/// A boolean that is true if the message is a tombstone.
pub const MESSAGING_KAFKA_MESSAGE_TOMBSTONE: &str = "messaging.kafka.message.tombstone";
/// The size of the message body in bytes.
///
/// This can refer to both the compressed or uncompressed body size. If both sizes are known, the uncompressed
/// body size should be used.
///
/// # Examples
///
/// - `1439`
pub const MESSAGING_MESSAGE_BODY_SIZE: &str = "messaging.message.body.size";
/// The conversation ID identifying the conversation to which the message belongs, represented as a string. Sometimes called &#34;Correlation ID&#34;.
///
/// # Examples
///
/// - `MyConversationId`
pub const MESSAGING_MESSAGE_CONVERSATION_ID: &str = "messaging.message.conversation_id";
/// The size of the message body and metadata in bytes.
///
/// This can refer to both the compressed or uncompressed size. If both sizes are known, the uncompressed
/// size should be used.
///
/// # Examples
///
/// - `2738`
pub const MESSAGING_MESSAGE_ENVELOPE_SIZE: &str = "messaging.message.envelope.size";
/// A value used by the messaging system as an identifier for the message, represented as a string.
///
/// # Examples
///
/// - `452a7c7c7c7048c2f887f61572b18fc2`
pub const MESSAGING_MESSAGE_ID: &str = "messaging.message.id";
/// Deprecated, use `messaging.operation.type` instead.
///
/// # Examples
///
/// - `publish`
/// - `create`
/// - `process`
#[deprecated]
pub const MESSAGING_OPERATION: &str = "messaging.operation";
/// The system-specific name of the messaging operation.
///
/// # Examples
///
/// - `ack`
/// - `nack`
/// - `send`
pub const MESSAGING_OPERATION_NAME: &str = "messaging.operation.name";
/// A string identifying the type of the messaging operation.
///
/// If a custom value is used, it MUST be of low cardinality.
pub const MESSAGING_OPERATION_TYPE: &str = "messaging.operation.type";
/// RabbitMQ message routing key.
///
/// # Examples
///
/// - `myKey`
pub const MESSAGING_RABBITMQ_DESTINATION_ROUTING_KEY: &str =
    "messaging.rabbitmq.destination.routing_key";
/// RabbitMQ message delivery tag.
///
/// # Examples
///
/// - `123`
pub const MESSAGING_RABBITMQ_MESSAGE_DELIVERY_TAG: &str = "messaging.rabbitmq.message.delivery_tag";
/// Name of the RocketMQ producer/consumer group that is handling the message. The client type is identified by the SpanKind.
///
/// # Examples
///
/// - `myConsumerGroup`
pub const MESSAGING_ROCKETMQ_CLIENT_GROUP: &str = "messaging.rocketmq.client_group";
/// Model of message consumption. This only applies to consumer spans.
pub const MESSAGING_ROCKETMQ_CONSUMPTION_MODEL: &str = "messaging.rocketmq.consumption_model";
/// The delay time level for delay message, which determines the message delay time.
///
/// # Examples
///
/// - `3`
pub const MESSAGING_ROCKETMQ_MESSAGE_DELAY_TIME_LEVEL: &str =
    "messaging.rocketmq.message.delay_time_level";
/// The timestamp in milliseconds that the delay message is expected to be delivered to consumer.
///
/// # Examples
///
/// - `1665987217045`
pub const MESSAGING_ROCKETMQ_MESSAGE_DELIVERY_TIMESTAMP: &str =
    "messaging.rocketmq.message.delivery_timestamp";
/// It is essential for FIFO message. Messages that belong to the same message group are always processed one by one within the same consumer group.
///
/// # Examples
///
/// - `myMessageGroup`
pub const MESSAGING_ROCKETMQ_MESSAGE_GROUP: &str = "messaging.rocketmq.message.group";
/// Key(s) of message, another way to mark message besides message id.
///
/// # Examples
///
/// - `keyA`
/// - `keyB`
pub const MESSAGING_ROCKETMQ_MESSAGE_KEYS: &str = "messaging.rocketmq.message.keys";
/// The secondary classifier of message besides topic.
///
/// # Examples
///
/// - `tagA`
pub const MESSAGING_ROCKETMQ_MESSAGE_TAG: &str = "messaging.rocketmq.message.tag";
/// Type of message.
pub const MESSAGING_ROCKETMQ_MESSAGE_TYPE: &str = "messaging.rocketmq.message.type";
/// Namespace of RocketMQ resources, resources in different namespaces are individual.
///
/// # Examples
///
/// - `myNamespace`
pub const MESSAGING_ROCKETMQ_NAMESPACE: &str = "messaging.rocketmq.namespace";
/// The name of the subscription in the topic messages are received from.
///
/// # Examples
///
/// - `mySubscription`
pub const MESSAGING_SERVICEBUS_DESTINATION_SUBSCRIPTION_NAME: &str =
    "messaging.servicebus.destination.subscription_name";
/// Describes the [settlement type](https://learn.microsoft.com/azure/service-bus-messaging/message-transfers-locks-settlement#peeklock).
pub const MESSAGING_SERVICEBUS_DISPOSITION_STATUS: &str = "messaging.servicebus.disposition_status";
/// Number of deliveries that have been attempted for this message.
///
/// # Examples
///
/// - `2`
pub const MESSAGING_SERVICEBUS_MESSAGE_DELIVERY_COUNT: &str =
    "messaging.servicebus.message.delivery_count";
/// The UTC epoch seconds at which the message has been accepted and stored in the entity.
///
/// # Examples
///
/// - `1701393730`
pub const MESSAGING_SERVICEBUS_MESSAGE_ENQUEUED_TIME: &str =
    "messaging.servicebus.message.enqueued_time";
/// The messaging system as identified by the client instrumentation.
///
/// The actual messaging system may differ from the one known by the client. For example, when using Kafka client libraries to communicate with Azure Event Hubs, the `messaging.system` is set to `kafka` based on the instrumentation&#39;s best knowledge.
pub const MESSAGING_SYSTEM: &str = "messaging.system";
/// Deprecated, use `network.local.address`.
///
/// # Examples
///
/// - `192.168.0.1`
#[deprecated]
pub const NET_HOST_IP: &str = "net.host.ip";
/// Deprecated, use `server.address`.
///
/// # Examples
///
/// - `example.com`
#[deprecated]
pub const NET_HOST_NAME: &str = "net.host.name";
/// Deprecated, use `server.port`.
///
/// # Examples
///
/// - `8080`
#[deprecated]
pub const NET_HOST_PORT: &str = "net.host.port";
/// Deprecated, use `network.peer.address`.
///
/// # Examples
///
/// - `127.0.0.1`
#[deprecated]
pub const NET_PEER_IP: &str = "net.peer.ip";
/// Deprecated, use `server.address` on client spans and `client.address` on server spans.
///
/// # Examples
///
/// - `example.com`
#[deprecated]
pub const NET_PEER_NAME: &str = "net.peer.name";
/// Deprecated, use `server.port` on client spans and `client.port` on server spans.
///
/// # Examples
///
/// - `8080`
#[deprecated]
pub const NET_PEER_PORT: &str = "net.peer.port";
/// Deprecated, use `network.protocol.name`.
///
/// # Examples
///
/// - `amqp`
/// - `http`
/// - `mqtt`
#[deprecated]
pub const NET_PROTOCOL_NAME: &str = "net.protocol.name";
/// Deprecated, use `network.protocol.version`.
///
/// # Examples
///
/// - `3.1.1`
#[deprecated]
pub const NET_PROTOCOL_VERSION: &str = "net.protocol.version";
/// Deprecated, use `network.transport` and `network.type`.
#[deprecated]
pub const NET_SOCK_FAMILY: &str = "net.sock.family";
/// Deprecated, use `network.local.address`.
///
/// # Examples
///
/// - `/var/my.sock`
#[deprecated]
pub const NET_SOCK_HOST_ADDR: &str = "net.sock.host.addr";
/// Deprecated, use `network.local.port`.
///
/// # Examples
///
/// - `8080`
#[deprecated]
pub const NET_SOCK_HOST_PORT: &str = "net.sock.host.port";
/// Deprecated, use `network.peer.address`.
///
/// # Examples
///
/// - `192.168.0.1`
#[deprecated]
pub const NET_SOCK_PEER_ADDR: &str = "net.sock.peer.addr";
/// Deprecated, no replacement at this time.
///
/// # Examples
///
/// - `/var/my.sock`
#[deprecated]
pub const NET_SOCK_PEER_NAME: &str = "net.sock.peer.name";
/// Deprecated, use `network.peer.port`.
///
/// # Examples
///
/// - `65531`
#[deprecated]
pub const NET_SOCK_PEER_PORT: &str = "net.sock.peer.port";
/// Deprecated, use `network.transport`.
#[deprecated]
pub const NET_TRANSPORT: &str = "net.transport";
/// The ISO 3166-1 alpha-2 2-character country code associated with the mobile carrier network.
///
/// # Examples
///
/// - `DE`
pub const NETWORK_CARRIER_ICC: &str = "network.carrier.icc";
/// The mobile carrier country code.
///
/// # Examples
///
/// - `310`
pub const NETWORK_CARRIER_MCC: &str = "network.carrier.mcc";
/// The mobile carrier network code.
///
/// # Examples
///
/// - `001`
pub const NETWORK_CARRIER_MNC: &str = "network.carrier.mnc";
/// The name of the mobile carrier.
///
/// # Examples
///
/// - `sprint`
pub const NETWORK_CARRIER_NAME: &str = "network.carrier.name";
/// This describes more details regarding the connection.type. It may be the type of cell technology connection, but it could be used for describing details about a wifi connection.
///
/// # Examples
///
/// - `LTE`
pub const NETWORK_CONNECTION_SUBTYPE: &str = "network.connection.subtype";
/// The internet connection type.
///
/// # Examples
///
/// - `wifi`
pub const NETWORK_CONNECTION_TYPE: &str = "network.connection.type";
/// The network IO operation direction.
///
/// # Examples
///
/// - `transmit`
pub const NETWORK_IO_DIRECTION: &str = "network.io.direction";
/// Local address of the network connection - IP address or Unix domain socket name.
///
/// # Examples
///
/// - `10.1.2.80`
/// - `/tmp/my.sock`
pub const NETWORK_LOCAL_ADDRESS: &str = "network.local.address";
/// Local port number of the network connection.
///
/// # Examples
///
/// - `65123`
pub const NETWORK_LOCAL_PORT: &str = "network.local.port";
/// Peer address of the database node where the operation was performed.
///
/// Semantic conventions for individual database systems SHOULD document whether `network.peer.*` attributes are applicable. Network peer address and port are useful when the application interacts with individual database nodes directly.
/// If a database operation involved multiple network calls (for example retries), the address of the last contacted node SHOULD be used.
///
/// # Examples
///
/// - `10.1.2.80`
/// - `/tmp/my.sock`
pub const NETWORK_PEER_ADDRESS: &str = "network.peer.address";
/// Peer port number of the network connection.
///
/// # Examples
///
/// - `65123`
pub const NETWORK_PEER_PORT: &str = "network.peer.port";
/// [OSI application layer](https://osi-model.com/application-layer/) or non-OSI equivalent.
///
/// The value SHOULD be normalized to lowercase.
///
/// # Examples
///
/// - `http`
/// - `spdy`
pub const NETWORK_PROTOCOL_NAME: &str = "network.protocol.name";
/// The actual version of the protocol used for network communication.
///
/// If protocol version is subject to negotiation (for example using [ALPN](https://www.rfc-editor.org/rfc/rfc7301.html)), this attribute SHOULD be set to the negotiated version. If the actual protocol version is not known, this attribute SHOULD NOT be set.
///
/// # Examples
///
/// - `1.0`
/// - `1.1`
/// - `2`
/// - `3`
pub const NETWORK_PROTOCOL_VERSION: &str = "network.protocol.version";
/// [OSI transport layer](https://osi-model.com/transport-layer/) or [inter-process communication method](https://wikipedia.org/wiki/Inter-process_communication).
///
/// The value SHOULD be normalized to lowercase.
///
/// Consider always setting the transport when setting a port number, since
/// a port number is ambiguous without knowing the transport. For example
/// different processes could be listening on TCP port 12345 and UDP port 12345.
///
/// # Examples
///
/// - `tcp`
/// - `unix`
pub const NETWORK_TRANSPORT: &str = "network.transport";
/// [OSI network layer](https://osi-model.com/network-layer/) or non-OSI equivalent.
///
/// The value SHOULD be normalized to lowercase.
///
/// # Examples
///
/// - `ipv4`
/// - `ipv6`
pub const NETWORK_TYPE: &str = "network.type";
/// The digest of the OCI image manifest. For container images specifically is the digest by which the container image is known.
///
/// Follows [OCI Image Manifest Specification](https://github.com/opencontainers/image-spec/blob/main/manifest.md), and specifically the [Digest property](https://github.com/opencontainers/image-spec/blob/main/descriptor.md#digests).
/// An example can be found in [Example Image Manifest](https://docs.docker.com/registry/spec/manifest-v2-2/#example-image-manifest).
///
/// # Examples
///
/// - `sha256:e4ca62c0d62f3e886e684806dfe9d4e0cda60d54986898173c1083856cfda0f4`
pub const OCI_MANIFEST_DIGEST: &str = "oci.manifest.digest";
/// Parent-child Reference type.
///
/// The causal relationship between a child Span and a parent Span.
pub const OPENTRACING_REF_TYPE: &str = "opentracing.ref_type";
/// Unique identifier for a particular build or compilation of the operating system.
///
/// # Examples
///
/// - `TQ3C.230805.001.B2`
/// - `20E247`
/// - `22621`
pub const OS_BUILD_ID: &str = "os.build_id";
/// Human readable (not intended to be parsed) OS version information, like e.g. reported by `ver` or `lsb_release -a` commands.
///
/// # Examples
///
/// - `Microsoft Windows [Version 10.0.18363.778]`
/// - `Ubuntu 18.04.1 LTS`
pub const OS_DESCRIPTION: &str = "os.description";
/// Human readable operating system name.
///
/// # Examples
///
/// - `iOS`
/// - `Android`
/// - `Ubuntu`
pub const OS_NAME: &str = "os.name";
/// The operating system type.
pub const OS_TYPE: &str = "os.type";
/// The version string of the operating system as defined in [Version Attributes](/docs/resource/README.md#version-attributes).
///
/// # Examples
///
/// - `14.2.1`
/// - `18.04.1`
pub const OS_VERSION: &str = "os.version";
/// None.
///
/// # Examples
///
/// - `io.opentelemetry.contrib.mongodb`
#[deprecated]
pub const OTEL_LIBRARY_NAME: &str = "otel.library.name";
/// None.
///
/// # Examples
///
/// - `1.0.0`
#[deprecated]
pub const OTEL_LIBRARY_VERSION: &str = "otel.library.version";
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
/// Name of the code, either &#34;OK&#34; or &#34;ERROR&#34;. MUST NOT be set if the status code is UNSET.
pub const OTEL_STATUS_CODE: &str = "otel.status_code";
/// Description of the Status if it has a value, otherwise not set.
///
/// # Examples
///
/// - `resource not found`
pub const OTEL_STATUS_DESCRIPTION: &str = "otel.status_description";
/// The [`service.name`](/docs/resource/README.md#service) of the remote service. SHOULD be equal to the actual `service.name` resource attribute of the remote service if any.
///
/// # Examples
///
/// - `AuthTokenCache`
pub const PEER_SERVICE: &str = "peer.service";
/// Deprecated, use `db.client.connections.pool.name` instead.
///
/// # Examples
///
/// - `myDataSource`
#[deprecated]
pub const POOL_NAME: &str = "pool.name";
/// The command used to launch the process (i.e. the command name). On Linux based systems, can be set to the zeroth string in `proc/[pid]/cmdline`. On Windows, can be set to the first parameter extracted from `GetCommandLineW`.
///
/// # Examples
///
/// - `cmd/otelcol`
pub const PROCESS_COMMAND: &str = "process.command";
/// All the command arguments (including the command/executable itself) as received by the process. On Linux-based systems (and some other Unixoid systems supporting procfs), can be set according to the list of null-delimited strings extracted from `proc/[pid]/cmdline`. For libc-based executables, this would be the full argv vector passed to `main`.
///
/// # Examples
///
/// - `cmd/otecol`
/// - `--config=config.yaml`
pub const PROCESS_COMMAND_ARGS: &str = "process.command_args";
/// The full command used to launch the process as a single string representing the full command. On Windows, can be set to the result of `GetCommandLineW`. Do not set this if you have to assemble it just for monitoring; use `process.command_args` instead.
///
/// # Examples
///
/// - `C:\cmd\otecol --config="my directory\config.yaml"`
pub const PROCESS_COMMAND_LINE: &str = "process.command_line";
/// Specifies whether the context switches for this data point were voluntary or involuntary.
pub const PROCESS_CONTEXT_SWITCH_TYPE: &str = "process.context_switch_type";
/// A process SHOULD be characterized _either_ by data points with no `state` labels, _or only_ data points with `state` labels.
pub const PROCESS_CPU_STATE: &str = "process.cpu.state";
/// The date and time the process was created, in ISO 8601 format.
///
/// # Examples
///
/// - `2023-11-21T09:25:34.853Z`
pub const PROCESS_CREATION_TIME: &str = "process.creation.time";
/// The name of the process executable. On Linux based systems, can be set to the `Name` in `proc/[pid]/status`. On Windows, can be set to the base name of `GetProcessImageFileNameW`.
///
/// # Examples
///
/// - `otelcol`
pub const PROCESS_EXECUTABLE_NAME: &str = "process.executable.name";
/// The full path to the process executable. On Linux based systems, can be set to the target of `proc/[pid]/exe`. On Windows, can be set to the result of `GetProcessImageFileNameW`.
///
/// # Examples
///
/// - `/usr/bin/cmd/otelcol`
pub const PROCESS_EXECUTABLE_PATH: &str = "process.executable.path";
/// The exit code of the process.
///
/// # Examples
///
/// - `127`
pub const PROCESS_EXIT_CODE: &str = "process.exit.code";
/// The date and time the process exited, in ISO 8601 format.
///
/// # Examples
///
/// - `2023-11-21T09:26:12.315Z`
pub const PROCESS_EXIT_TIME: &str = "process.exit.time";
/// The PID of the process&#39;s group leader. This is also the process group ID (PGID) of the process.
///
/// # Examples
///
/// - `23`
pub const PROCESS_GROUP_LEADER_PID: &str = "process.group_leader.pid";
/// Whether the process is connected to an interactive shell.
pub const PROCESS_INTERACTIVE: &str = "process.interactive";
/// The username of the user that owns the process.
///
/// # Examples
///
/// - `root`
pub const PROCESS_OWNER: &str = "process.owner";
/// The type of page fault for this data point. Type `major` is for major/hard page faults, and `minor` is for minor/soft page faults.
pub const PROCESS_PAGING_FAULT_TYPE: &str = "process.paging.fault_type";
/// Parent Process identifier (PPID).
///
/// # Examples
///
/// - `111`
pub const PROCESS_PARENT_PID: &str = "process.parent_pid";
/// Process identifier (PID).
///
/// # Examples
///
/// - `1234`
pub const PROCESS_PID: &str = "process.pid";
/// The real user ID (RUID) of the process.
///
/// # Examples
///
/// - `1000`
pub const PROCESS_REAL_USER_ID: &str = "process.real_user.id";
/// The username of the real user of the process.
///
/// # Examples
///
/// - `operator`
pub const PROCESS_REAL_USER_NAME: &str = "process.real_user.name";
/// An additional description about the runtime of the process, for example a specific vendor customization of the runtime environment.
///
/// # Examples
///
/// - `Eclipse OpenJ9 Eclipse OpenJ9 VM openj9-0.21.0`
pub const PROCESS_RUNTIME_DESCRIPTION: &str = "process.runtime.description";
/// The name of the runtime of this process. For compiled native binaries, this SHOULD be the name of the compiler.
///
/// # Examples
///
/// - `OpenJDK Runtime Environment`
pub const PROCESS_RUNTIME_NAME: &str = "process.runtime.name";
/// The version of the runtime of this process, as returned by the runtime without modification.
///
/// # Examples
///
/// - `14.0.2`
pub const PROCESS_RUNTIME_VERSION: &str = "process.runtime.version";
/// The saved user ID (SUID) of the process.
///
/// # Examples
///
/// - `1002`
pub const PROCESS_SAVED_USER_ID: &str = "process.saved_user.id";
/// The username of the saved user.
///
/// # Examples
///
/// - `operator`
pub const PROCESS_SAVED_USER_NAME: &str = "process.saved_user.name";
/// The PID of the process&#39;s session leader. This is also the session ID (SID) of the process.
///
/// # Examples
///
/// - `14`
pub const PROCESS_SESSION_LEADER_PID: &str = "process.session_leader.pid";
/// The effective user ID (EUID) of the process.
///
/// # Examples
///
/// - `1001`
pub const PROCESS_USER_ID: &str = "process.user.id";
/// The username of the effective user of the process.
///
/// # Examples
///
/// - `root`
pub const PROCESS_USER_NAME: &str = "process.user.name";
/// Virtual process identifier.
///
/// The process ID within a PID namespace. This is not necessarily unique across all processes on the host but it is unique within the process namespace that the process exists within.
///
/// # Examples
///
/// - `12`
pub const PROCESS_VPID: &str = "process.vpid";
/// The [error codes](https://connect.build/docs/protocol/#error-codes) of the Connect request. Error codes are always string values.
pub const RPC_CONNECT_RPC_ERROR_CODE: &str = "rpc.connect_rpc.error_code";
/// The [numeric status code](https://github.com/grpc/grpc/blob/v1.33.2/doc/statuscodes.md) of the gRPC request.
pub const RPC_GRPC_STATUS_CODE: &str = "rpc.grpc.status_code";
/// `error.code` property of response if it is an error response.
///
/// # Examples
///
/// - `-32700`
/// - `100`
pub const RPC_JSONRPC_ERROR_CODE: &str = "rpc.jsonrpc.error_code";
/// `error.message` property of response if it is an error response.
///
/// # Examples
///
/// - `Parse error`
/// - `User already exists`
pub const RPC_JSONRPC_ERROR_MESSAGE: &str = "rpc.jsonrpc.error_message";
/// `id` property of request or response. Since protocol allows id to be int, string, `null` or missing (for notifications), value is expected to be cast to string for simplicity. Use empty string in case of `null` value. Omit entirely if this is a notification.
///
/// # Examples
///
/// - `10`
/// - `request-7`
/// - ``
pub const RPC_JSONRPC_REQUEST_ID: &str = "rpc.jsonrpc.request_id";
/// Protocol version as in `jsonrpc` property of request/response. Since JSON-RPC 1.0 doesn&#39;t specify this, the value can be omitted.
///
/// # Examples
///
/// - `2.0`
/// - `1.0`
pub const RPC_JSONRPC_VERSION: &str = "rpc.jsonrpc.version";
/// Compressed size of the message in bytes.
pub const RPC_MESSAGE_COMPRESSED_SIZE: &str = "rpc.message.compressed_size";
/// MUST be calculated as two different counters starting from `1` one for sent messages and one for received message.
///
/// This way we guarantee that the values will be consistent between different implementations.
pub const RPC_MESSAGE_ID: &str = "rpc.message.id";
/// Whether this is a received or sent message.
pub const RPC_MESSAGE_TYPE: &str = "rpc.message.type";
/// Uncompressed size of the message in bytes.
pub const RPC_MESSAGE_UNCOMPRESSED_SIZE: &str = "rpc.message.uncompressed_size";
/// The name of the (logical) method being called, must be equal to the $method part in the span name.
///
/// This is the logical name of the method from the RPC interface perspective, which can be different from the name of any implementing method/function. The `code.function` attribute may be used to store the latter (e.g., method actually executing the call on the server side, RPC client stub method on the client side).
///
/// # Examples
///
/// - `exampleMethod`
pub const RPC_METHOD: &str = "rpc.method";
/// The full (logical) name of the service being called, including its package name, if applicable.
///
/// This is the logical name of the service from the RPC interface perspective, which can be different from the name of any implementing class. The `code.namespace` attribute may be used to store the latter (despite the attribute name, it may include a class name; e.g., class with method actually executing the call on the server side, RPC client stub class on the client side).
///
/// # Examples
///
/// - `myservice.EchoService`
pub const RPC_SERVICE: &str = "rpc.service";
/// A string identifying the remoting system. See below for a list of well-known identifiers.
pub const RPC_SYSTEM: &str = "rpc.system";
/// Name of the database host.
///
/// When observed from the client side, and when communicating through an intermediary, `server.address` SHOULD represent the server address behind any intermediaries, for example proxies, if it&#39;s available.
///
/// # Examples
///
/// - `example.com`
/// - `10.1.2.80`
/// - `/tmp/my.sock`
pub const SERVER_ADDRESS: &str = "server.address";
/// Server port number.
///
/// When observed from the client side, and when communicating through an intermediary, `server.port` SHOULD represent the server port behind any intermediaries, for example proxies, if it&#39;s available.
///
/// # Examples
///
/// - `80`
/// - `8080`
/// - `443`
pub const SERVER_PORT: &str = "server.port";
/// The string ID of the service instance.
///
/// MUST be unique for each instance of the same `service.namespace,service.name` pair (in other words
/// `service.namespace,service.name,service.instance.id` triplet MUST be globally unique). The ID helps to
/// distinguish instances of the same service that exist at the same time (e.g. instances of a horizontally scaled
/// service).
///
/// Implementations, such as SDKs, are recommended to generate a random Version 1 or Version 4 [RFC
/// 4122](https://www.ietf.org/rfc/rfc4122.txt) UUID, but are free to use an inherent unique ID as the source of
/// this value if stability is desirable. In that case, the ID SHOULD be used as source of a UUID Version 5 and
/// SHOULD use the following UUID as the namespace: `4d63009a-8d0f-11ee-aad7-4c796ed8e320`.
///
/// UUIDs are typically recommended, as only an opaque value for the purposes of identifying a service instance is
/// needed. Similar to what can be seen in the man page for the
/// [`/etc/machine-id`](https://www.freedesktop.org/software/systemd/man/machine-id.html) file, the underlying
/// data, such as pod name and namespace should be treated as confidential, being the user&#39;s choice to expose it
/// or not via another resource attribute.
///
/// For applications running behind an application server (like unicorn), we do not recommend using one identifier
/// for all processes participating in the application. Instead, it&#39;s recommended each division (e.g. a worker
/// thread in unicorn) to have its own instance.id.
///
/// It&#39;s not recommended for a Collector to set `service.instance.id` if it can&#39;t unambiguously determine the
/// service instance that is generating that telemetry. For instance, creating an UUID based on `pod.name` will
/// likely be wrong, as the Collector might not know from which container within that pod the telemetry originated.
/// However, Collectors can set the `service.instance.id` if they can unambiguously determine the service instance
/// for that telemetry. This is typically the case for scraping receivers, as they know the target address and
/// port.
///
/// # Examples
///
/// - `627cc493-f310-47de-96bd-71410b7dec09`
pub const SERVICE_INSTANCE_ID: &str = "service.instance.id";
/// Logical name of the service.
///
/// MUST be the same for all instances of horizontally scaled services. If the value was not specified, SDKs MUST fallback to `unknown_service:` concatenated with [`process.executable.name`](process.md), e.g. `unknown_service:bash`. If `process.executable.name` is not available, the value MUST be set to `unknown_service`.
///
/// # Examples
///
/// - `shoppingcart`
pub const SERVICE_NAME: &str = "service.name";
/// A namespace for `service.name`.
///
/// A string value having a meaning that helps to distinguish a group of services, for example the team name that owns a group of services. `service.name` is expected to be unique within the same namespace. If `service.namespace` is not specified in the Resource then `service.name` is expected to be unique for all services that have no explicit namespace defined (so the empty/unspecified namespace is simply one more valid namespace). Zero-length namespace string is assumed equal to unspecified namespace.
///
/// # Examples
///
/// - `Shop`
pub const SERVICE_NAMESPACE: &str = "service.namespace";
/// The version string of the service API or implementation. The format is not defined by these conventions.
///
/// # Examples
///
/// - `2.0.0`
/// - `a01dbef8a`
pub const SERVICE_VERSION: &str = "service.version";
/// A unique id to identify a session.
///
/// # Examples
///
/// - `00112233-4455-6677-8899-aabbccddeeff`
pub const SESSION_ID: &str = "session.id";
/// The previous `session.id` for this user, when known.
///
/// # Examples
///
/// - `00112233-4455-6677-8899-aabbccddeeff`
pub const SESSION_PREVIOUS_ID: &str = "session.previous_id";
/// SignalR HTTP connection closure status.
///
/// # Examples
///
/// - `app_shutdown`
/// - `timeout`
pub const SIGNALR_CONNECTION_STATUS: &str = "signalr.connection.status";
/// [SignalR transport type](https://github.com/dotnet/aspnetcore/blob/main/src/SignalR/docs/specs/TransportProtocols.md).
///
/// # Examples
///
/// - `web_sockets`
/// - `long_polling`
pub const SIGNALR_TRANSPORT: &str = "signalr.transport";
/// Source address - domain name if available without reverse DNS lookup; otherwise, IP address or Unix domain socket name.
///
/// When observed from the destination side, and when communicating through an intermediary, `source.address` SHOULD represent the source address behind any intermediaries, for example proxies, if it&#39;s available.
///
/// # Examples
///
/// - `source.example.com`
/// - `10.1.2.80`
/// - `/tmp/my.sock`
pub const SOURCE_ADDRESS: &str = "source.address";
/// Source port number.
///
/// # Examples
///
/// - `3389`
/// - `2888`
pub const SOURCE_PORT: &str = "source.port";
/// Deprecated, use `db.client.connections.state` instead.
///
/// # Examples
///
/// - `idle`
#[deprecated]
pub const STATE: &str = "state";
/// The logical CPU number [0..n-1].
///
/// # Examples
///
/// - `1`
pub const SYSTEM_CPU_LOGICAL_NUMBER: &str = "system.cpu.logical_number";
/// The CPU state for this data point. A system&#39;s CPU SHOULD be characterized *either* by data points with no `state` labels, *or only* data points with `state` labels.
///
/// # Examples
///
/// - `idle`
/// - `interrupt`
pub const SYSTEM_CPU_STATE: &str = "system.cpu.state";
/// The device identifier.
///
/// # Examples
///
/// - `(identifier)`
pub const SYSTEM_DEVICE: &str = "system.device";
/// The filesystem mode.
///
/// # Examples
///
/// - `rw, ro`
pub const SYSTEM_FILESYSTEM_MODE: &str = "system.filesystem.mode";
/// The filesystem mount path.
///
/// # Examples
///
/// - `/mnt/data`
pub const SYSTEM_FILESYSTEM_MOUNTPOINT: &str = "system.filesystem.mountpoint";
/// The filesystem state.
///
/// # Examples
///
/// - `used`
pub const SYSTEM_FILESYSTEM_STATE: &str = "system.filesystem.state";
/// The filesystem type.
///
/// # Examples
///
/// - `ext4`
pub const SYSTEM_FILESYSTEM_TYPE: &str = "system.filesystem.type";
/// The memory state.
///
/// # Examples
///
/// - `free`
/// - `cached`
pub const SYSTEM_MEMORY_STATE: &str = "system.memory.state";
/// A stateless protocol MUST NOT set this attribute.
///
/// # Examples
///
/// - `close_wait`
pub const SYSTEM_NETWORK_STATE: &str = "system.network.state";
/// The paging access direction.
///
/// # Examples
///
/// - `in`
pub const SYSTEM_PAGING_DIRECTION: &str = "system.paging.direction";
/// The memory paging state.
///
/// # Examples
///
/// - `free`
pub const SYSTEM_PAGING_STATE: &str = "system.paging.state";
/// The memory paging type.
///
/// # Examples
///
/// - `minor`
pub const SYSTEM_PAGING_TYPE: &str = "system.paging.type";
/// The process state, e.g., [Linux Process State Codes](https://man7.org/linux/man-pages/man1/ps.1.html#PROCESS_STATE_CODES).
///
/// # Examples
///
/// - `running`
pub const SYSTEM_PROCESS_STATUS: &str = "system.process.status";
/// Deprecated, use `system.process.status` instead.
///
/// # Examples
///
/// - `running`
#[deprecated]
pub const SYSTEM_PROCESSES_STATUS: &str = "system.processes.status";
/// The name of the auto instrumentation agent or distribution, if used.
///
/// Official auto instrumentation agents and distributions SHOULD set the `telemetry.distro.name` attribute to
/// a string starting with `opentelemetry-`, e.g. `opentelemetry-java-instrumentation`.
///
/// # Examples
///
/// - `parts-unlimited-java`
pub const TELEMETRY_DISTRO_NAME: &str = "telemetry.distro.name";
/// The version string of the auto instrumentation agent or distribution, if used.
///
/// # Examples
///
/// - `1.2.3`
pub const TELEMETRY_DISTRO_VERSION: &str = "telemetry.distro.version";
/// The language of the telemetry SDK.
pub const TELEMETRY_SDK_LANGUAGE: &str = "telemetry.sdk.language";
/// The name of the telemetry SDK as defined above.
///
/// The OpenTelemetry SDK MUST set the `telemetry.sdk.name` attribute to `opentelemetry`.
/// If another SDK, like a fork or a vendor-provided implementation, is used, this SDK MUST set the
/// `telemetry.sdk.name` attribute to the fully-qualified class or module name of this SDK&#39;s main entry point
/// or another suitable identifier depending on the language.
/// The identifier `opentelemetry` is reserved and MUST NOT be used in this case.
/// All custom identifiers SHOULD be stable across different versions of an implementation.
///
/// # Examples
///
/// - `opentelemetry`
pub const TELEMETRY_SDK_NAME: &str = "telemetry.sdk.name";
/// The version string of the telemetry SDK.
///
/// # Examples
///
/// - `1.2.3`
pub const TELEMETRY_SDK_VERSION: &str = "telemetry.sdk.version";
/// Current &#34;managed&#34; thread ID (as opposed to OS thread ID).
///
/// # Examples
///
/// - `42`
pub const THREAD_ID: &str = "thread.id";
/// Current thread name.
///
/// # Examples
///
/// - `main`
pub const THREAD_NAME: &str = "thread.name";
/// String indicating the [cipher](https://datatracker.ietf.org/doc/html/rfc5246#appendix-A.5) used during the current connection.
///
/// The values allowed for `tls.cipher` MUST be one of the `Descriptions` of the [registered TLS Cipher Suits](https://www.iana.org/assignments/tls-parameters/tls-parameters.xhtml#table-tls-parameters-4).
///
/// # Examples
///
/// - `TLS_RSA_WITH_3DES_EDE_CBC_SHA`
/// - `TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256`
pub const TLS_CIPHER: &str = "tls.cipher";
/// PEM-encoded stand-alone certificate offered by the client. This is usually mutually-exclusive of `client.certificate_chain` since this value also exists in that list.
///
/// # Examples
///
/// - `MII...`
pub const TLS_CLIENT_CERTIFICATE: &str = "tls.client.certificate";
/// Array of PEM-encoded certificates that make up the certificate chain offered by the client. This is usually mutually-exclusive of `client.certificate` since that value should be the first certificate in the chain.
///
/// # Examples
///
/// - `MII...`
/// - `MI...`
pub const TLS_CLIENT_CERTIFICATE_CHAIN: &str = "tls.client.certificate_chain";
/// Certificate fingerprint using the MD5 digest of DER-encoded version of certificate offered by the client. For consistency with other hash values, this value should be formatted as an uppercase hash.
///
/// # Examples
///
/// - `0F76C7F2C55BFD7D8E8B8F4BFBF0C9EC`
pub const TLS_CLIENT_HASH_MD5: &str = "tls.client.hash.md5";
/// Certificate fingerprint using the SHA1 digest of DER-encoded version of certificate offered by the client. For consistency with other hash values, this value should be formatted as an uppercase hash.
///
/// # Examples
///
/// - `9E393D93138888D288266C2D915214D1D1CCEB2A`
pub const TLS_CLIENT_HASH_SHA1: &str = "tls.client.hash.sha1";
/// Certificate fingerprint using the SHA256 digest of DER-encoded version of certificate offered by the client. For consistency with other hash values, this value should be formatted as an uppercase hash.
///
/// # Examples
///
/// - `0687F666A054EF17A08E2F2162EAB4CBC0D265E1D7875BE74BF3C712CA92DAF0`
pub const TLS_CLIENT_HASH_SHA256: &str = "tls.client.hash.sha256";
/// Distinguished name of [subject](https://datatracker.ietf.org/doc/html/rfc5280#section-4.1.2.6) of the issuer of the x.509 certificate presented by the client.
///
/// # Examples
///
/// - `CN=Example Root CA, OU=Infrastructure Team, DC=example, DC=com`
pub const TLS_CLIENT_ISSUER: &str = "tls.client.issuer";
/// A hash that identifies clients based on how they perform an SSL/TLS handshake.
///
/// # Examples
///
/// - `d4e5b18d6b55c71272893221c96ba240`
pub const TLS_CLIENT_JA3: &str = "tls.client.ja3";
/// Date/Time indicating when client certificate is no longer considered valid.
///
/// # Examples
///
/// - `2021-01-01T00:00:00.000Z`
pub const TLS_CLIENT_NOT_AFTER: &str = "tls.client.not_after";
/// Date/Time indicating when client certificate is first considered valid.
///
/// # Examples
///
/// - `1970-01-01T00:00:00.000Z`
pub const TLS_CLIENT_NOT_BEFORE: &str = "tls.client.not_before";
/// Also called an SNI, this tells the server which hostname to which the client is attempting to connect to.
///
/// # Examples
///
/// - `opentelemetry.io`
pub const TLS_CLIENT_SERVER_NAME: &str = "tls.client.server_name";
/// Distinguished name of subject of the x.509 certificate presented by the client.
///
/// # Examples
///
/// - `CN=myclient, OU=Documentation Team, DC=example, DC=com`
pub const TLS_CLIENT_SUBJECT: &str = "tls.client.subject";
/// Array of ciphers offered by the client during the client hello.
///
/// # Examples
///
/// - `"TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384", "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384", "..."`
pub const TLS_CLIENT_SUPPORTED_CIPHERS: &str = "tls.client.supported_ciphers";
/// String indicating the curve used for the given cipher, when applicable.
///
/// # Examples
///
/// - `secp256r1`
pub const TLS_CURVE: &str = "tls.curve";
/// Boolean flag indicating if the TLS negotiation was successful and transitioned to an encrypted tunnel.
///
/// # Examples
///
/// - `True`
pub const TLS_ESTABLISHED: &str = "tls.established";
/// String indicating the protocol being tunneled. Per the values in the [IANA registry](https://www.iana.org/assignments/tls-extensiontype-values/tls-extensiontype-values.xhtml#alpn-protocol-ids), this string should be lower case.
///
/// # Examples
///
/// - `http/1.1`
pub const TLS_NEXT_PROTOCOL: &str = "tls.next_protocol";
/// Normalized lowercase protocol name parsed from original string of the negotiated [SSL/TLS protocol version](https://www.openssl.org/docs/man1.1.1/man3/SSL_get_version.html#RETURN-VALUES).
pub const TLS_PROTOCOL_NAME: &str = "tls.protocol.name";
/// Numeric part of the version parsed from the original string of the negotiated [SSL/TLS protocol version](https://www.openssl.org/docs/man1.1.1/man3/SSL_get_version.html#RETURN-VALUES).
///
/// # Examples
///
/// - `1.2`
/// - `3`
pub const TLS_PROTOCOL_VERSION: &str = "tls.protocol.version";
/// Boolean flag indicating if this TLS connection was resumed from an existing TLS negotiation.
///
/// # Examples
///
/// - `True`
pub const TLS_RESUMED: &str = "tls.resumed";
/// PEM-encoded stand-alone certificate offered by the server. This is usually mutually-exclusive of `server.certificate_chain` since this value also exists in that list.
///
/// # Examples
///
/// - `MII...`
pub const TLS_SERVER_CERTIFICATE: &str = "tls.server.certificate";
/// Array of PEM-encoded certificates that make up the certificate chain offered by the server. This is usually mutually-exclusive of `server.certificate` since that value should be the first certificate in the chain.
///
/// # Examples
///
/// - `MII...`
/// - `MI...`
pub const TLS_SERVER_CERTIFICATE_CHAIN: &str = "tls.server.certificate_chain";
/// Certificate fingerprint using the MD5 digest of DER-encoded version of certificate offered by the server. For consistency with other hash values, this value should be formatted as an uppercase hash.
///
/// # Examples
///
/// - `0F76C7F2C55BFD7D8E8B8F4BFBF0C9EC`
pub const TLS_SERVER_HASH_MD5: &str = "tls.server.hash.md5";
/// Certificate fingerprint using the SHA1 digest of DER-encoded version of certificate offered by the server. For consistency with other hash values, this value should be formatted as an uppercase hash.
///
/// # Examples
///
/// - `9E393D93138888D288266C2D915214D1D1CCEB2A`
pub const TLS_SERVER_HASH_SHA1: &str = "tls.server.hash.sha1";
/// Certificate fingerprint using the SHA256 digest of DER-encoded version of certificate offered by the server. For consistency with other hash values, this value should be formatted as an uppercase hash.
///
/// # Examples
///
/// - `0687F666A054EF17A08E2F2162EAB4CBC0D265E1D7875BE74BF3C712CA92DAF0`
pub const TLS_SERVER_HASH_SHA256: &str = "tls.server.hash.sha256";
/// Distinguished name of [subject](https://datatracker.ietf.org/doc/html/rfc5280#section-4.1.2.6) of the issuer of the x.509 certificate presented by the client.
///
/// # Examples
///
/// - `CN=Example Root CA, OU=Infrastructure Team, DC=example, DC=com`
pub const TLS_SERVER_ISSUER: &str = "tls.server.issuer";
/// A hash that identifies servers based on how they perform an SSL/TLS handshake.
///
/// # Examples
///
/// - `d4e5b18d6b55c71272893221c96ba240`
pub const TLS_SERVER_JA3S: &str = "tls.server.ja3s";
/// Date/Time indicating when server certificate is no longer considered valid.
///
/// # Examples
///
/// - `2021-01-01T00:00:00.000Z`
pub const TLS_SERVER_NOT_AFTER: &str = "tls.server.not_after";
/// Date/Time indicating when server certificate is first considered valid.
///
/// # Examples
///
/// - `1970-01-01T00:00:00.000Z`
pub const TLS_SERVER_NOT_BEFORE: &str = "tls.server.not_before";
/// Distinguished name of subject of the x.509 certificate presented by the server.
///
/// # Examples
///
/// - `CN=myserver, OU=Documentation Team, DC=example, DC=com`
pub const TLS_SERVER_SUBJECT: &str = "tls.server.subject";
/// Domain extracted from the `url.full`, such as &#34;opentelemetry.io&#34;.
///
/// In some cases a URL may refer to an IP and/or port directly, without a domain name. In this case, the IP address would go to the domain field. If the URL contains a [literal IPv6 address](https://www.rfc-editor.org/rfc/rfc2732#section-2) enclosed by `[` and `]`, the `[` and `]` characters should also be captured in the domain field.
///
/// # Examples
///
/// - `www.foo.bar`
/// - `opentelemetry.io`
/// - `3.12.167.2`
/// - `[1080:0:0:0:8:800:200C:417A]`
pub const URL_DOMAIN: &str = "url.domain";
/// The file extension extracted from the `url.full`, excluding the leading dot.
///
/// The file extension is only set if it exists, as not every url has a file extension. When the file name has multiple extensions `example.tar.gz`, only the last one should be captured `gz`, not `tar.gz`.
///
/// # Examples
///
/// - `png`
/// - `gz`
pub const URL_EXTENSION: &str = "url.extension";
/// The [URI fragment](https://www.rfc-editor.org/rfc/rfc3986#section-3.5) component.
///
/// # Examples
///
/// - `SemConv`
pub const URL_FRAGMENT: &str = "url.fragment";
/// Absolute URL describing a network resource according to [RFC3986](https://www.rfc-editor.org/rfc/rfc3986).
///
/// For network calls, URL usually has `scheme://host[:port][path][?query][#fragment]` format, where the fragment is not transmitted over HTTP, but if it is known, it SHOULD be included nevertheless.
/// `url.full` MUST NOT contain credentials passed via URL in form of `https://username:password@www.example.com/`. In such case username and password SHOULD be redacted and attribute&#39;s value SHOULD be `https://REDACTED:REDACTED@www.example.com/`.
/// `url.full` SHOULD capture the absolute URL when it is available (or can be reconstructed). Sensitive content provided in `url.full` SHOULD be scrubbed when instrumentations can identify it.
///
/// # Examples
///
/// - `https://www.foo.bar/search?q=OpenTelemetry#SemConv`
/// - `//localhost`
pub const URL_FULL: &str = "url.full";
/// Unmodified original URL as seen in the event source.
///
/// In network monitoring, the observed URL may be a full URL, whereas in access logs, the URL is often just represented as a path. This field is meant to represent the URL as it was observed, complete or not.
/// `url.original` might contain credentials passed via URL in form of `https://username:password@www.example.com/`. In such case password and username SHOULD NOT be redacted and attribute&#39;s value SHOULD remain the same.
///
/// # Examples
///
/// - `https://www.foo.bar/search?q=OpenTelemetry#SemConv`
/// - `search?q=OpenTelemetry`
pub const URL_ORIGINAL: &str = "url.original";
/// The [URI path](https://www.rfc-editor.org/rfc/rfc3986#section-3.3) component.
///
/// Sensitive content provided in `url.path` SHOULD be scrubbed when instrumentations can identify it.
///
/// # Examples
///
/// - `/search`
pub const URL_PATH: &str = "url.path";
/// Port extracted from the `url.full`.
///
/// # Examples
///
/// - `443`
pub const URL_PORT: &str = "url.port";
/// The [URI query](https://www.rfc-editor.org/rfc/rfc3986#section-3.4) component.
///
/// Sensitive content provided in `url.query` SHOULD be scrubbed when instrumentations can identify it.
///
/// # Examples
///
/// - `q=OpenTelemetry`
pub const URL_QUERY: &str = "url.query";
/// The highest registered url domain, stripped of the subdomain.
///
/// This value can be determined precisely with the [public suffix list](http://publicsuffix.org). For example, the registered domain for `foo.example.com` is `example.com`. Trying to approximate this by simply taking the last two labels will not work well for TLDs such as `co.uk`.
///
/// # Examples
///
/// - `example.com`
/// - `foo.co.uk`
pub const URL_REGISTERED_DOMAIN: &str = "url.registered_domain";
/// The [URI scheme](https://www.rfc-editor.org/rfc/rfc3986#section-3.1) component identifying the used protocol.
///
/// # Examples
///
/// - `http`
/// - `https`
pub const URL_SCHEME: &str = "url.scheme";
/// The subdomain portion of a fully qualified domain name includes all of the names except the host name under the registered_domain. In a partially qualified domain, or if the qualification level of the full name cannot be determined, subdomain contains all of the names below the registered domain.
///
/// The subdomain portion of `www.east.mydomain.co.uk` is `east`. If the domain has multiple levels of subdomain, such as `sub2.sub1.example.com`, the subdomain field should contain `sub2.sub1`, with no trailing period.
///
/// # Examples
///
/// - `east`
/// - `sub2.sub1`
pub const URL_SUBDOMAIN: &str = "url.subdomain";
/// The low-cardinality template of an [absolute path reference](https://www.rfc-editor.org/rfc/rfc3986#section-4.2).
///
/// The `url.template` MUST have low cardinality. It is not usually available on HTTP clients, but may be known by the application or specialized HTTP instrumentation.
///
/// # Examples
///
/// - `/users/{id}`
/// - `/users/:id`
/// - `/users?id={id}`
pub const URL_TEMPLATE: &str = "url.template";
/// The effective top level domain (eTLD), also known as the domain suffix, is the last part of the domain name. For example, the top level domain for example.com is `com`.
///
/// This value can be determined precisely with the [public suffix list](http://publicsuffix.org).
///
/// # Examples
///
/// - `com`
/// - `co.uk`
pub const URL_TOP_LEVEL_DOMAIN: &str = "url.top_level_domain";
/// Name of the user-agent extracted from original. Usually refers to the browser&#39;s name.
///
/// [Example](https://www.whatsmyua.info) of extracting browser&#39;s name from original string. In the case of using a user-agent for non-browser products, such as microservices with multiple names/versions inside the `user_agent.original`, the most significant name SHOULD be selected. In such a scenario it should align with `user_agent.version`
///
/// # Examples
///
/// - `Safari`
/// - `YourApp`
pub const USER_AGENT_NAME: &str = "user_agent.name";
/// Value of the [HTTP User-Agent](https://www.rfc-editor.org/rfc/rfc9110.html#field.user-agent) header sent by the client.
///
/// # Examples
///
/// - `CERN-LineMode/2.15 libwww/2.17b3`
/// - `Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.2 Mobile/15E148 Safari/604.1`
/// - `YourApp/1.0.0 grpc-java-okhttp/1.27.2`
pub const USER_AGENT_ORIGINAL: &str = "user_agent.original";
/// Version of the user-agent extracted from original. Usually refers to the browser&#39;s version.
///
/// [Example](https://www.whatsmyua.info) of extracting browser&#39;s version from original string. In the case of using a user-agent for non-browser products, such as microservices with multiple names/versions inside the `user_agent.original`, the most significant version SHOULD be selected. In such a scenario it should align with `user_agent.name`
///
/// # Examples
///
/// - `14.1.2`
/// - `1.0.0`
pub const USER_AGENT_VERSION: &str = "user_agent.version";
/// Additional description of the web engine (e.g. detailed version and edition information).
///
/// # Examples
///
/// - `WildFly Full 21.0.0.Final (WildFly Core 13.0.1.Final) - 2.2.2.Final`
pub const WEBENGINE_DESCRIPTION: &str = "webengine.description";
/// The name of the web engine.
///
/// # Examples
///
/// - `WildFly`
pub const WEBENGINE_NAME: &str = "webengine.name";
/// The version of the web engine.
///
/// # Examples
///
/// - `21.0.0`
pub const WEBENGINE_VERSION: &str = "webengine.version";
