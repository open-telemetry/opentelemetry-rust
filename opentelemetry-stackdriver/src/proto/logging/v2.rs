/// An individual entry in a log.
///
///
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogEntry {
    /// Required. The resource name of the log to which this log entry belongs:
    ///
    ///     "projects/\[PROJECT_ID]/logs/[LOG_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/logs/[LOG_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/logs/[LOG_ID\]"
    ///     "folders/\[FOLDER_ID]/logs/[LOG_ID\]"
    ///
    /// A project number may be used in place of PROJECT_ID. The project number is
    /// translated to its corresponding PROJECT_ID internally and the `log_name`
    /// field will contain PROJECT_ID in queries and exports.
    ///
    /// `\[LOG_ID\]` must be URL-encoded within `log_name`. Example:
    /// `"organizations/1234567890/logs/cloudresourcemanager.googleapis.com%2Factivity"`.
    /// `\[LOG_ID\]` must be less than 512 characters long and can only include the
    /// following characters: upper and lower case alphanumeric characters,
    /// forward-slash, underscore, hyphen, and period.
    ///
    /// For backward compatibility, if `log_name` begins with a forward-slash, such
    /// as `/projects/...`, then the log entry is ingested as usual but the
    /// forward-slash is removed. Listing the log entry will not show the leading
    /// slash and filtering for a log name with a leading slash will never return
    /// any results.
    #[prost(string, tag = "12")]
    pub log_name: ::prost::alloc::string::String,
    /// Required. The monitored resource that produced this log entry.
    ///
    /// Example: a log entry that reports a database error would be associated with
    /// the monitored resource designating the particular database that reported
    /// the error.
    #[prost(message, optional, tag = "8")]
    pub resource: ::core::option::Option<super::super::api::MonitoredResource>,
    /// Optional. The time the event described by the log entry occurred. This time is used
    /// to compute the log entry's age and to enforce the logs retention period.
    /// If this field is omitted in a new log entry, then Logging assigns it the
    /// current time. Timestamps have nanosecond accuracy, but trailing zeros in
    /// the fractional seconds might be omitted when the timestamp is displayed.
    ///
    /// Incoming log entries must have timestamps that don't exceed the
    /// [logs retention
    /// period](<https://cloud.google.com/logging/quotas#logs_retention_periods>) in
    /// the past, and that don't exceed 24 hours in the future. Log entries outside
    /// those time boundaries aren't ingested by Logging.
    #[prost(message, optional, tag = "9")]
    pub timestamp: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. The time the log entry was received by Logging.
    #[prost(message, optional, tag = "24")]
    pub receive_timestamp: ::core::option::Option<::prost_types::Timestamp>,
    /// Optional. The severity of the log entry. The default value is `LogSeverity.DEFAULT`.
    #[prost(enumeration = "super::r#type::LogSeverity", tag = "10")]
    pub severity: i32,
    /// Optional. A unique identifier for the log entry. If you provide a value, then
    /// Logging considers other log entries in the same project, with the same
    /// `timestamp`, and with the same `insert_id` to be duplicates which are
    /// removed in a single query result. However, there are no guarantees of
    /// de-duplication in the export of logs.
    ///
    /// If the `insert_id` is omitted when writing a log entry, the Logging API
    ///  assigns its own unique identifier in this field.
    ///
    /// In queries, the `insert_id` is also used to order log entries that have
    /// the same `log_name` and `timestamp` values.
    #[prost(string, tag = "4")]
    pub insert_id: ::prost::alloc::string::String,
    /// Optional. Information about the HTTP request associated with this log entry, if
    /// applicable.
    #[prost(message, optional, tag = "7")]
    pub http_request: ::core::option::Option<super::r#type::HttpRequest>,
    /// Optional. A set of user-defined (key, value) data that provides additional
    /// information about the log entry.
    #[prost(map = "string, string", tag = "11")]
    pub labels:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Optional. Information about an operation associated with the log entry, if
    /// applicable.
    #[prost(message, optional, tag = "15")]
    pub operation: ::core::option::Option<LogEntryOperation>,
    /// Optional. Resource name of the trace associated with the log entry, if any. If it
    /// contains a relative resource name, the name is assumed to be relative to
    /// `//tracing.googleapis.com`. Example:
    /// `projects/my-projectid/traces/06796866738c859f2f19b7cfb3214824`
    #[prost(string, tag = "22")]
    pub trace: ::prost::alloc::string::String,
    /// Optional. The span ID within the trace associated with the log entry.
    ///
    /// For Trace spans, this is the same format that the Trace API v2 uses: a
    /// 16-character hexadecimal encoding of an 8-byte array, such as
    /// `000000000000004a`.
    #[prost(string, tag = "27")]
    pub span_id: ::prost::alloc::string::String,
    /// Optional. The sampling decision of the trace associated with the log entry.
    ///
    /// True means that the trace resource name in the `trace` field was sampled
    /// for storage in a trace backend. False means that the trace was not sampled
    /// for storage when this log entry was written, or the sampling decision was
    /// unknown at the time. A non-sampled `trace` value is still useful as a
    /// request correlation identifier. The default is False.
    #[prost(bool, tag = "30")]
    pub trace_sampled: bool,
    /// Optional. Source code location information associated with the log entry, if any.
    #[prost(message, optional, tag = "23")]
    pub source_location: ::core::option::Option<LogEntrySourceLocation>,
    /// The log entry payload, which can be one of multiple types.
    #[prost(oneof = "log_entry::Payload", tags = "2, 3, 6")]
    pub payload: ::core::option::Option<log_entry::Payload>,
}
/// Nested message and enum types in `LogEntry`.
pub mod log_entry {
    /// The log entry payload, which can be one of multiple types.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        /// The log entry payload, represented as a protocol buffer. Some Google
        /// Cloud Platform services use this field for their log entry payloads.
        ///
        /// The following protocol buffer types are supported; user-defined types
        /// are not supported:
        ///
        ///   "type.googleapis.com/google.cloud.audit.AuditLog"
        ///   "type.googleapis.com/google.appengine.logging.v1.RequestLog"
        #[prost(message, tag = "2")]
        ProtoPayload(::prost_types::Any),
        /// The log entry payload, represented as a Unicode string (UTF-8).
        #[prost(string, tag = "3")]
        TextPayload(::prost::alloc::string::String),
        /// The log entry payload, represented as a structure that is
        /// expressed as a JSON object.
        #[prost(message, tag = "6")]
        JsonPayload(::prost_types::Struct),
    }
}
/// Additional information about a potentially long-running operation with which
/// a log entry is associated.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogEntryOperation {
    /// Optional. An arbitrary operation identifier. Log entries with the same
    /// identifier are assumed to be part of the same operation.
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    /// Optional. An arbitrary producer identifier. The combination of `id` and
    /// `producer` must be globally unique. Examples for `producer`:
    /// `"MyDivision.MyBigCompany.com"`, `"github.com/MyProject/MyApplication"`.
    #[prost(string, tag = "2")]
    pub producer: ::prost::alloc::string::String,
    /// Optional. Set this to True if this is the first log entry in the operation.
    #[prost(bool, tag = "3")]
    pub first: bool,
    /// Optional. Set this to True if this is the last log entry in the operation.
    #[prost(bool, tag = "4")]
    pub last: bool,
}
/// Additional information about the source code location that produced the log
/// entry.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogEntrySourceLocation {
    /// Optional. Source file name. Depending on the runtime environment, this
    /// might be a simple name or a fully-qualified name.
    #[prost(string, tag = "1")]
    pub file: ::prost::alloc::string::String,
    /// Optional. Line within the source file. 1-based; 0 indicates no line number
    /// available.
    #[prost(int64, tag = "2")]
    pub line: i64,
    /// Optional. Human-readable name of the function or method being invoked, with
    /// optional context such as the class or package name. This information may be
    /// used in contexts such as the logs viewer, where a file and line number are
    /// less meaningful. The format can vary by language. For example:
    /// `qual.if.ied.Class.method` (Java), `dir/package.func` (Go), `function`
    /// (Python).
    #[prost(string, tag = "3")]
    pub function: ::prost::alloc::string::String,
}
/// Describes a repository of logs.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogBucket {
    /// The resource name of the bucket.
    /// For example:
    /// "projects/my-project-id/locations/my-location/buckets/my-bucket-id The
    /// supported locations are:
    ///   "global"
    ///
    /// For the location of `global` it is unspecified where logs are actually
    /// stored.
    /// Once a bucket has been created, the location can not be changed.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Describes this bucket.
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    /// Output only. The creation timestamp of the bucket. This is not set for any of the
    /// default buckets.
    #[prost(message, optional, tag = "4")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. The last update timestamp of the bucket.
    #[prost(message, optional, tag = "5")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Logs will be retained by default for this amount of time, after which they
    /// will automatically be deleted. The minimum retention period is 1 day.
    /// If this value is set to zero at bucket creation time, the default time of
    /// 30 days will be used.
    #[prost(int32, tag = "11")]
    pub retention_days: i32,
    /// Whether the bucket has been locked.
    /// The retention period on a locked bucket may not be changed.
    /// Locked buckets may only be deleted if they are empty.
    #[prost(bool, tag = "9")]
    pub locked: bool,
    /// Output only. The bucket lifecycle state.
    #[prost(enumeration = "LifecycleState", tag = "12")]
    pub lifecycle_state: i32,
}
/// Describes a view over logs in a bucket.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogView {
    /// The resource name of the view.
    /// For example
    /// "projects/my-project-id/locations/my-location/buckets/my-bucket-id/views/my-view
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Describes this view.
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    /// Output only. The creation timestamp of the view.
    #[prost(message, optional, tag = "4")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. The last update timestamp of the view.
    #[prost(message, optional, tag = "5")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Filter that restricts which log entries in a bucket are visible in this
    /// view. Filters are restricted to be a logical AND of ==/!= of any of the
    /// following:
    ///   originating project/folder/organization/billing account.
    ///   resource type
    ///   log id
    /// Example: SOURCE("projects/myproject") AND resource.type = "gce_instance"
    ///             AND LOG_ID("stdout")
    #[prost(string, tag = "7")]
    pub filter: ::prost::alloc::string::String,
}
/// Describes a sink used to export log entries to one of the following
/// destinations in any project: a Cloud Storage bucket, a BigQuery dataset, or a
/// Cloud Pub/Sub topic. A logs filter controls which log entries are exported.
/// The sink must be created within a project, organization, billing account, or
/// folder.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogSink {
    /// Required. The client-assigned sink identifier, unique within the project. Example:
    /// `"my-syslog-errors-to-pubsub"`. Sink identifiers are limited to 100
    /// characters and can include only the following characters: upper and
    /// lower-case alphanumeric characters, underscores, hyphens, and periods.
    /// First character has to be alphanumeric.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Required. The export destination:
    ///
    ///     "storage.googleapis.com/\[GCS_BUCKET\]"
    ///     "bigquery.googleapis.com/projects/\[PROJECT_ID]/datasets/[DATASET\]"
    ///     "pubsub.googleapis.com/projects/\[PROJECT_ID]/topics/[TOPIC_ID\]"
    ///
    /// The sink's `writer_identity`, set when the sink is created, must
    /// have permission to write to the destination or else the log
    /// entries are not exported. For more information, see
    /// [Exporting Logs with
    /// Sinks](<https://cloud.google.com/logging/docs/api/tasks/exporting-logs>).
    #[prost(string, tag = "3")]
    pub destination: ::prost::alloc::string::String,
    /// Optional. An [advanced logs
    /// filter](<https://cloud.google.com/logging/docs/view/advanced-queries>). The
    /// only exported log entries are those that are in the resource owning the
    /// sink and that match the filter. For example:
    ///
    ///     logName="projects/\[PROJECT_ID]/logs/[LOG_ID\]" AND severity>=ERROR
    #[prost(string, tag = "5")]
    pub filter: ::prost::alloc::string::String,
    /// Optional. A description of this sink.
    /// The maximum length of the description is 8000 characters.
    #[prost(string, tag = "18")]
    pub description: ::prost::alloc::string::String,
    /// Optional. If set to True, then this sink is disabled and it does not
    /// export any log entries.
    #[prost(bool, tag = "19")]
    pub disabled: bool,
    /// Optional. Log entries that match any of the exclusion filters will not be exported.
    /// If a log entry is matched by both `filter` and one of `exclusion_filters`
    /// it will not be exported.
    #[prost(message, repeated, tag = "16")]
    pub exclusions: ::prost::alloc::vec::Vec<LogExclusion>,
    /// Deprecated. This field is unused.
    #[deprecated]
    #[prost(enumeration = "log_sink::VersionFormat", tag = "6")]
    pub output_version_format: i32,
    /// Output only. An IAM identity&mdash;a service account or group&mdash;under which Logging
    /// writes the exported log entries to the sink's destination. This field is
    /// set by \[sinks.create][google.logging.v2.ConfigServiceV2.CreateSink\] and
    /// \[sinks.update][google.logging.v2.ConfigServiceV2.UpdateSink\] based on the
    /// value of `unique_writer_identity` in those methods.
    ///
    /// Until you grant this identity write-access to the destination, log entry
    /// exports from this sink will fail. For more information,
    /// see [Granting Access for a
    /// Resource](<https://cloud.google.com/iam/docs/granting-roles-to-service-accounts#granting_access_to_a_service_account_for_a_resource>).
    /// Consult the destination service's documentation to determine the
    /// appropriate IAM roles to assign to the identity.
    #[prost(string, tag = "8")]
    pub writer_identity: ::prost::alloc::string::String,
    /// Optional. This field applies only to sinks owned by organizations and
    /// folders. If the field is false, the default, only the logs owned by the
    /// sink's parent resource are available for export. If the field is true, then
    /// logs from all the projects, folders, and billing accounts contained in the
    /// sink's parent resource are also available for export. Whether a particular
    /// log entry from the children is exported depends on the sink's filter
    /// expression. For example, if this field is true, then the filter
    /// `resource.type=gce_instance` would export all Compute Engine VM instance
    /// log entries from all projects in the sink's parent. To only export entries
    /// from certain child projects, filter on the project part of the log name:
    ///
    ///     logName:("projects/test-project1/" OR "projects/test-project2/") AND
    ///     resource.type=gce_instance
    #[prost(bool, tag = "9")]
    pub include_children: bool,
    /// Output only. The creation timestamp of the sink.
    ///
    /// This field may not be present for older sinks.
    #[prost(message, optional, tag = "13")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. The last update timestamp of the sink.
    ///
    /// This field may not be present for older sinks.
    #[prost(message, optional, tag = "14")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Destination dependent options.
    #[prost(oneof = "log_sink::Options", tags = "12")]
    pub options: ::core::option::Option<log_sink::Options>,
}
/// Nested message and enum types in `LogSink`.
pub mod log_sink {
    /// Deprecated. This is unused.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum VersionFormat {
        /// An unspecified format version that will default to V2.
        Unspecified = 0,
        /// `LogEntry` version 2 format.
        V2 = 1,
        /// `LogEntry` version 1 format.
        V1 = 2,
    }
    /// Destination dependent options.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Options {
        /// Optional. Options that affect sinks exporting data to BigQuery.
        #[prost(message, tag = "12")]
        BigqueryOptions(super::BigQueryOptions),
    }
}
/// Options that change functionality of a sink exporting data to BigQuery.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BigQueryOptions {
    /// Optional. Whether to use [BigQuery's partition
    /// tables](<https://cloud.google.com/bigquery/docs/partitioned-tables>). By
    /// default, Logging creates dated tables based on the log entries' timestamps,
    /// e.g. syslog_20170523. With partitioned tables the date suffix is no longer
    /// present and [special query
    /// syntax](<https://cloud.google.com/bigquery/docs/querying-partitioned-tables>)
    /// has to be used instead. In both cases, tables are sharded based on UTC
    /// timezone.
    #[prost(bool, tag = "1")]
    pub use_partitioned_tables: bool,
    /// Output only. True if new timestamp column based partitioning is in use,
    /// false if legacy ingestion-time partitioning is in use.
    /// All new sinks will have this field set true and will use timestamp column
    /// based partitioning. If use_partitioned_tables is false, this value has no
    /// meaning and will be false. Legacy sinks using partitioned tables will have
    /// this field set to false.
    #[prost(bool, tag = "3")]
    pub uses_timestamp_column_partitioning: bool,
}
/// The parameters to `ListBuckets`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBucketsRequest {
    /// Required. The parent resource whose buckets are to be listed:
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/locations/[LOCATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID\]"
    ///     "folders/\[FOLDER_ID]/locations/[LOCATION_ID\]"
    ///
    /// Note: The locations portion of the resource must be specified, but
    /// supplying the character `-` in place of \[LOCATION_ID\] will return all
    /// buckets.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Optional. If present, then retrieve the next batch of results from the
    /// preceding call to this method. `pageToken` must be the value of
    /// `nextPageToken` from the previous response. The values of other method
    /// parameters should be identical to those in the previous call.
    #[prost(string, tag = "2")]
    pub page_token: ::prost::alloc::string::String,
    /// Optional. The maximum number of results to return from this request.
    /// Non-positive values are ignored. The presence of `nextPageToken` in the
    /// response indicates that more results might be available.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
}
/// The response from ListBuckets.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBucketsResponse {
    /// A list of buckets.
    #[prost(message, repeated, tag = "1")]
    pub buckets: ::prost::alloc::vec::Vec<LogBucket>,
    /// If there might be more results than appear in this response, then
    /// `nextPageToken` is included. To get the next set of results, call the same
    /// method again using the value of `nextPageToken` as `pageToken`.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The parameters to `CreateBucket`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateBucketRequest {
    /// Required. The resource in which to create the bucket:
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID\]"
    ///
    /// Example: `"projects/my-logging-project/locations/global"`
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. A client-assigned identifier such as `"my-bucket"`. Identifiers are
    /// limited to 100 characters and can include only letters, digits,
    /// underscores, hyphens, and periods.
    #[prost(string, tag = "2")]
    pub bucket_id: ::prost::alloc::string::String,
    /// Required. The new bucket. The region specified in the new bucket must be compliant
    /// with any Location Restriction Org Policy. The name field in the bucket is
    /// ignored.
    #[prost(message, optional, tag = "3")]
    pub bucket: ::core::option::Option<LogBucket>,
}
/// The parameters to `UpdateBucket`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateBucketRequest {
    /// Required. The full resource name of the bucket to update.
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///
    /// Example:
    /// `"projects/my-project-id/locations/my-location/buckets/my-bucket-id"`. Also
    /// requires permission "resourcemanager.projects.updateLiens" to set the
    /// locked property
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Required. The updated bucket.
    #[prost(message, optional, tag = "2")]
    pub bucket: ::core::option::Option<LogBucket>,
    /// Required. Field mask that specifies the fields in `bucket` that need an update. A
    /// bucket field will be overwritten if, and only if, it is in the update
    /// mask. `name` and output only fields cannot be updated.
    ///
    /// For a detailed `FieldMask` definition, see
    /// <https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.FieldMask>
    ///
    /// Example: `updateMask=retention_days`.
    #[prost(message, optional, tag = "4")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// The parameters to `GetBucket`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBucketRequest {
    /// Required. The resource name of the bucket:
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///
    /// Example:
    /// `"projects/my-project-id/locations/my-location/buckets/my-bucket-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The parameters to `DeleteBucket`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteBucketRequest {
    /// Required. The full resource name of the bucket to delete.
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///
    /// Example:
    /// `"projects/my-project-id/locations/my-location/buckets/my-bucket-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The parameters to `UndeleteBucket`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UndeleteBucketRequest {
    /// Required. The full resource name of the bucket to undelete.
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///     "folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///
    /// Example:
    /// `"projects/my-project-id/locations/my-location/buckets/my-bucket-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The parameters to `ListViews`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListViewsRequest {
    /// Required. The bucket whose views are to be listed:
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Optional. If present, then retrieve the next batch of results from the
    /// preceding call to this method. `pageToken` must be the value of
    /// `nextPageToken` from the previous response. The values of other method
    /// parameters should be identical to those in the previous call.
    #[prost(string, tag = "2")]
    pub page_token: ::prost::alloc::string::String,
    /// Optional. The maximum number of results to return from this request.
    /// Non-positive values are ignored. The presence of `nextPageToken` in the
    /// response indicates that more results might be available.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
}
/// The response from ListViews.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListViewsResponse {
    /// A list of views.
    #[prost(message, repeated, tag = "1")]
    pub views: ::prost::alloc::vec::Vec<LogView>,
    /// If there might be more results than appear in this response, then
    /// `nextPageToken` is included. To get the next set of results, call the same
    /// method again using the value of `nextPageToken` as `pageToken`.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The parameters to `CreateView`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateViewRequest {
    /// Required. The bucket in which to create the view
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID\]"
    ///
    /// Example:
    /// `"projects/my-logging-project/locations/my-location/buckets/my-bucket"`
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. The id to use for this view.
    #[prost(string, tag = "2")]
    pub view_id: ::prost::alloc::string::String,
    /// Required. The new view.
    #[prost(message, optional, tag = "3")]
    pub view: ::core::option::Option<LogView>,
}
/// The parameters to `UpdateView`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateViewRequest {
    /// Required. The full resource name of the view to update
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]"
    ///
    /// Example:
    ///   `"projects/my-project-id/locations/my-location/buckets/my-bucket-id/views/my-view-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Required. The updated view.
    #[prost(message, optional, tag = "2")]
    pub view: ::core::option::Option<LogView>,
    /// Optional. Field mask that specifies the fields in `view` that need
    /// an update. A field will be overwritten if, and only if, it is
    /// in the update mask. `name` and output only fields cannot be updated.
    ///
    /// For a detailed `FieldMask` definition, see
    /// <https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.FieldMask>
    ///
    /// Example: `updateMask=filter`.
    #[prost(message, optional, tag = "4")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// The parameters to `GetView`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetViewRequest {
    /// Required. The resource name of the policy:
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]"
    ///
    /// Example:
    /// `"projects/my-project-id/locations/my-location/buckets/my-bucket-id/views/my-view-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The parameters to `DeleteView`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteViewRequest {
    /// Required. The full resource name of the view to delete:
    ///
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]"
    ///
    /// Example:
    ///    `"projects/my-project-id/locations/my-location/buckets/my-bucket-id/views/my-view-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The parameters to `ListSinks`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSinksRequest {
    /// Required. The parent resource whose sinks are to be listed:
    ///
    ///     "projects/\[PROJECT_ID\]"
    ///     "organizations/\[ORGANIZATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]"
    ///     "folders/\[FOLDER_ID\]"
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Optional. If present, then retrieve the next batch of results from the
    /// preceding call to this method. `pageToken` must be the value of
    /// `nextPageToken` from the previous response. The values of other method
    /// parameters should be identical to those in the previous call.
    #[prost(string, tag = "2")]
    pub page_token: ::prost::alloc::string::String,
    /// Optional. The maximum number of results to return from this request.
    /// Non-positive values are ignored. The presence of `nextPageToken` in the
    /// response indicates that more results might be available.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
}
/// Result returned from `ListSinks`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSinksResponse {
    /// A list of sinks.
    #[prost(message, repeated, tag = "1")]
    pub sinks: ::prost::alloc::vec::Vec<LogSink>,
    /// If there might be more results than appear in this response, then
    /// `nextPageToken` is included. To get the next set of results, call the same
    /// method again using the value of `nextPageToken` as `pageToken`.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The parameters to `GetSink`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSinkRequest {
    /// Required. The resource name of the sink:
    ///
    ///     "projects/\[PROJECT_ID]/sinks/[SINK_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/sinks/[SINK_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/sinks/[SINK_ID\]"
    ///     "folders/\[FOLDER_ID]/sinks/[SINK_ID\]"
    ///
    /// Example: `"projects/my-project-id/sinks/my-sink-id"`.
    #[prost(string, tag = "1")]
    pub sink_name: ::prost::alloc::string::String,
}
/// The parameters to `CreateSink`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSinkRequest {
    /// Required. The resource in which to create the sink:
    ///
    ///     "projects/\[PROJECT_ID\]"
    ///     "organizations/\[ORGANIZATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]"
    ///     "folders/\[FOLDER_ID\]"
    ///
    /// Examples: `"projects/my-logging-project"`, `"organizations/123456789"`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. The new sink, whose `name` parameter is a sink identifier that
    /// is not already in use.
    #[prost(message, optional, tag = "2")]
    pub sink: ::core::option::Option<LogSink>,
    /// Optional. Determines the kind of IAM identity returned as `writer_identity`
    /// in the new sink. If this value is omitted or set to false, and if the
    /// sink's parent is a project, then the value returned as `writer_identity` is
    /// the same group or service account used by Logging before the addition of
    /// writer identities to this API. The sink's destination must be in the same
    /// project as the sink itself.
    ///
    /// If this field is set to true, or if the sink is owned by a non-project
    /// resource such as an organization, then the value of `writer_identity` will
    /// be a unique service account used only for exports from the new sink. For
    /// more information, see `writer_identity` in \[LogSink][google.logging.v2.LogSink\].
    #[prost(bool, tag = "3")]
    pub unique_writer_identity: bool,
}
/// The parameters to `UpdateSink`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateSinkRequest {
    /// Required. The full resource name of the sink to update, including the parent
    /// resource and the sink identifier:
    ///
    ///     "projects/\[PROJECT_ID]/sinks/[SINK_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/sinks/[SINK_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/sinks/[SINK_ID\]"
    ///     "folders/\[FOLDER_ID]/sinks/[SINK_ID\]"
    ///
    /// Example: `"projects/my-project-id/sinks/my-sink-id"`.
    #[prost(string, tag = "1")]
    pub sink_name: ::prost::alloc::string::String,
    /// Required. The updated sink, whose name is the same identifier that appears as part
    /// of `sink_name`.
    #[prost(message, optional, tag = "2")]
    pub sink: ::core::option::Option<LogSink>,
    /// Optional. See \[sinks.create][google.logging.v2.ConfigServiceV2.CreateSink\]
    /// for a description of this field. When updating a sink, the effect of this
    /// field on the value of `writer_identity` in the updated sink depends on both
    /// the old and new values of this field:
    ///
    /// +   If the old and new values of this field are both false or both true,
    ///     then there is no change to the sink's `writer_identity`.
    /// +   If the old value is false and the new value is true, then
    ///     `writer_identity` is changed to a unique service account.
    /// +   It is an error if the old value is true and the new value is
    ///     set to false or defaulted to false.
    #[prost(bool, tag = "3")]
    pub unique_writer_identity: bool,
    /// Optional. Field mask that specifies the fields in `sink` that need
    /// an update. A sink field will be overwritten if, and only if, it is
    /// in the update mask. `name` and output only fields cannot be updated.
    ///
    /// An empty updateMask is temporarily treated as using the following mask
    /// for backwards compatibility purposes:
    ///   destination,filter,includeChildren
    /// At some point in the future, behavior will be removed and specifying an
    /// empty updateMask will be an error.
    ///
    /// For a detailed `FieldMask` definition, see
    /// <https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.FieldMask>
    ///
    /// Example: `updateMask=filter`.
    #[prost(message, optional, tag = "4")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// The parameters to `DeleteSink`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteSinkRequest {
    /// Required. The full resource name of the sink to delete, including the parent
    /// resource and the sink identifier:
    ///
    ///     "projects/\[PROJECT_ID]/sinks/[SINK_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/sinks/[SINK_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/sinks/[SINK_ID\]"
    ///     "folders/\[FOLDER_ID]/sinks/[SINK_ID\]"
    ///
    /// Example: `"projects/my-project-id/sinks/my-sink-id"`.
    #[prost(string, tag = "1")]
    pub sink_name: ::prost::alloc::string::String,
}
/// Specifies a set of log entries that are not to be stored in
/// Logging. If your GCP resource receives a large volume of logs, you can
/// use exclusions to reduce your chargeable logs. Exclusions are
/// processed after log sinks, so you can export log entries before they are
/// excluded. Note that organization-level and folder-level exclusions don't
/// apply to child resources, and that you can't exclude audit log entries.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogExclusion {
    /// Required. A client-assigned identifier, such as `"load-balancer-exclusion"`.
    /// Identifiers are limited to 100 characters and can include only letters,
    /// digits, underscores, hyphens, and periods. First character has to be
    /// alphanumeric.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Optional. A description of this exclusion.
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// Required. An [advanced logs
    /// filter](<https://cloud.google.com/logging/docs/view/advanced-queries>) that
    /// matches the log entries to be excluded. By using the [sample
    /// function](<https://cloud.google.com/logging/docs/view/advanced-queries#sample>),
    /// you can exclude less than 100% of the matching log entries.
    /// For example, the following query matches 99% of low-severity log
    /// entries from Google Cloud Storage buckets:
    ///
    /// `"resource.type=gcs_bucket severity<ERROR sample(insertId, 0.99)"`
    #[prost(string, tag = "3")]
    pub filter: ::prost::alloc::string::String,
    /// Optional. If set to True, then this exclusion is disabled and it does not
    /// exclude any log entries. You can [update an
    /// exclusion]\[google.logging.v2.ConfigServiceV2.UpdateExclusion\] to change the
    /// value of this field.
    #[prost(bool, tag = "4")]
    pub disabled: bool,
    /// Output only. The creation timestamp of the exclusion.
    ///
    /// This field may not be present for older exclusions.
    #[prost(message, optional, tag = "5")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. The last update timestamp of the exclusion.
    ///
    /// This field may not be present for older exclusions.
    #[prost(message, optional, tag = "6")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// The parameters to `ListExclusions`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListExclusionsRequest {
    /// Required. The parent resource whose exclusions are to be listed.
    ///
    ///     "projects/\[PROJECT_ID\]"
    ///     "organizations/\[ORGANIZATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]"
    ///     "folders/\[FOLDER_ID\]"
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Optional. If present, then retrieve the next batch of results from the
    /// preceding call to this method. `pageToken` must be the value of
    /// `nextPageToken` from the previous response. The values of other method
    /// parameters should be identical to those in the previous call.
    #[prost(string, tag = "2")]
    pub page_token: ::prost::alloc::string::String,
    /// Optional. The maximum number of results to return from this request.
    /// Non-positive values are ignored. The presence of `nextPageToken` in the
    /// response indicates that more results might be available.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
}
/// Result returned from `ListExclusions`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListExclusionsResponse {
    /// A list of exclusions.
    #[prost(message, repeated, tag = "1")]
    pub exclusions: ::prost::alloc::vec::Vec<LogExclusion>,
    /// If there might be more results than appear in this response, then
    /// `nextPageToken` is included. To get the next set of results, call the same
    /// method again using the value of `nextPageToken` as `pageToken`.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The parameters to `GetExclusion`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetExclusionRequest {
    /// Required. The resource name of an existing exclusion:
    ///
    ///     "projects/\[PROJECT_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "folders/\[FOLDER_ID]/exclusions/[EXCLUSION_ID\]"
    ///
    /// Example: `"projects/my-project-id/exclusions/my-exclusion-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The parameters to `CreateExclusion`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateExclusionRequest {
    /// Required. The parent resource in which to create the exclusion:
    ///
    ///     "projects/\[PROJECT_ID\]"
    ///     "organizations/\[ORGANIZATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]"
    ///     "folders/\[FOLDER_ID\]"
    ///
    /// Examples: `"projects/my-logging-project"`, `"organizations/123456789"`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. The new exclusion, whose `name` parameter is an exclusion name
    /// that is not already used in the parent resource.
    #[prost(message, optional, tag = "2")]
    pub exclusion: ::core::option::Option<LogExclusion>,
}
/// The parameters to `UpdateExclusion`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateExclusionRequest {
    /// Required. The resource name of the exclusion to update:
    ///
    ///     "projects/\[PROJECT_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "folders/\[FOLDER_ID]/exclusions/[EXCLUSION_ID\]"
    ///
    /// Example: `"projects/my-project-id/exclusions/my-exclusion-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Required. New values for the existing exclusion. Only the fields specified in
    /// `update_mask` are relevant.
    #[prost(message, optional, tag = "2")]
    pub exclusion: ::core::option::Option<LogExclusion>,
    /// Required. A non-empty list of fields to change in the existing exclusion. New values
    /// for the fields are taken from the corresponding fields in the
    /// \[LogExclusion][google.logging.v2.LogExclusion\] included in this request. Fields not mentioned in
    /// `update_mask` are not changed and are ignored in the request.
    ///
    /// For example, to change the filter and description of an exclusion,
    /// specify an `update_mask` of `"filter,description"`.
    #[prost(message, optional, tag = "3")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// The parameters to `DeleteExclusion`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteExclusionRequest {
    /// Required. The resource name of an existing exclusion to delete:
    ///
    ///     "projects/\[PROJECT_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/exclusions/[EXCLUSION_ID\]"
    ///     "folders/\[FOLDER_ID]/exclusions/[EXCLUSION_ID\]"
    ///
    /// Example: `"projects/my-project-id/exclusions/my-exclusion-id"`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The parameters to
/// \[GetCmekSettings][google.logging.v2.ConfigServiceV2.GetCmekSettings\].
///
/// See [Enabling CMEK for Logs
/// Router](<https://cloud.google.com/logging/docs/routing/managed-encryption>) for
/// more information.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCmekSettingsRequest {
    /// Required. The resource for which to retrieve CMEK settings.
    ///
    ///     "projects/\[PROJECT_ID\]/cmekSettings"
    ///     "organizations/\[ORGANIZATION_ID\]/cmekSettings"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]/cmekSettings"
    ///     "folders/\[FOLDER_ID\]/cmekSettings"
    ///
    /// Example: `"organizations/12345/cmekSettings"`.
    ///
    /// Note: CMEK for the Logs Router can currently only be configured for GCP
    /// organizations. Once configured, it applies to all projects and folders in
    /// the GCP organization.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The parameters to
/// \[UpdateCmekSettings][google.logging.v2.ConfigServiceV2.UpdateCmekSettings\].
///
/// See [Enabling CMEK for Logs
/// Router](<https://cloud.google.com/logging/docs/routing/managed-encryption>) for
/// more information.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCmekSettingsRequest {
    /// Required. The resource name for the CMEK settings to update.
    ///
    ///     "projects/\[PROJECT_ID\]/cmekSettings"
    ///     "organizations/\[ORGANIZATION_ID\]/cmekSettings"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]/cmekSettings"
    ///     "folders/\[FOLDER_ID\]/cmekSettings"
    ///
    /// Example: `"organizations/12345/cmekSettings"`.
    ///
    /// Note: CMEK for the Logs Router can currently only be configured for GCP
    /// organizations. Once configured, it applies to all projects and folders in
    /// the GCP organization.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Required. The CMEK settings to update.
    ///
    /// See [Enabling CMEK for Logs
    /// Router](<https://cloud.google.com/logging/docs/routing/managed-encryption>)
    /// for more information.
    #[prost(message, optional, tag = "2")]
    pub cmek_settings: ::core::option::Option<CmekSettings>,
    /// Optional. Field mask identifying which fields from `cmek_settings` should
    /// be updated. A field will be overwritten if and only if it is in the update
    /// mask. Output only fields cannot be updated.
    ///
    /// See \[FieldMask][google.protobuf.FieldMask\] for more information.
    ///
    /// Example: `"updateMask=kmsKeyName"`
    #[prost(message, optional, tag = "3")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// Describes the customer-managed encryption key (CMEK) settings associated with
/// a project, folder, organization, billing account, or flexible resource.
///
/// Note: CMEK for the Logs Router can currently only be configured for GCP
/// organizations. Once configured, it applies to all projects and folders in the
/// GCP organization.
///
/// See [Enabling CMEK for Logs
/// Router](<https://cloud.google.com/logging/docs/routing/managed-encryption>) for
/// more information.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CmekSettings {
    /// Output only. The resource name of the CMEK settings.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The resource name for the configured Cloud KMS key.
    ///
    /// KMS key name format:
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION]/keyRings/[KEYRING]/cryptoKeys/[KEY\]"
    ///
    /// For example:
    ///     `"projects/my-project-id/locations/my-region/keyRings/key-ring-name/cryptoKeys/key-name"`
    ///
    ///
    ///
    /// To enable CMEK for the Logs Router, set this field to a valid
    /// `kms_key_name` for which the associated service account has the required
    /// `roles/cloudkms.cryptoKeyEncrypterDecrypter` role assigned for the key.
    ///
    /// The Cloud KMS key used by the Log Router can be updated by changing the
    /// `kms_key_name` to a new valid key name. Encryption operations that are in
    /// progress will be completed with the key that was in use when they started.
    /// Decryption operations will be completed using the key that was used at the
    /// time of encryption unless access to that key has been revoked.
    ///
    /// To disable CMEK for the Logs Router, set this field to an empty string.
    ///
    /// See [Enabling CMEK for Logs
    /// Router](<https://cloud.google.com/logging/docs/routing/managed-encryption>)
    /// for more information.
    #[prost(string, tag = "2")]
    pub kms_key_name: ::prost::alloc::string::String,
    /// Output only. The service account that will be used by the Logs Router to access your
    /// Cloud KMS key.
    ///
    /// Before enabling CMEK for Logs Router, you must first assign the role
    /// `roles/cloudkms.cryptoKeyEncrypterDecrypter` to the service account that
    /// the Logs Router will use to access your Cloud KMS key. Use
    /// \[GetCmekSettings][google.logging.v2.ConfigServiceV2.GetCmekSettings\] to
    /// obtain the service account ID.
    ///
    /// See [Enabling CMEK for Logs
    /// Router](<https://cloud.google.com/logging/docs/routing/managed-encryption>)
    /// for more information.
    #[prost(string, tag = "3")]
    pub service_account_id: ::prost::alloc::string::String,
}
/// LogBucket lifecycle states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum LifecycleState {
    /// Unspecified state.  This is only used/useful for distinguishing
    /// unset values.
    Unspecified = 0,
    /// The normal and active state.
    Active = 1,
    /// The bucket has been marked for deletion by the user.
    DeleteRequested = 2,
}
#[doc = r" Generated client implementations."]
pub mod config_service_v2_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = " Service for configuring sinks used to route log entries."]
    #[derive(Debug, Clone)]
    pub struct ConfigServiceV2Client<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ConfigServiceV2Client<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ConfigServiceV2Client<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ConfigServiceV2Client<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            ConfigServiceV2Client::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        #[doc = " Lists buckets."]
        pub async fn list_buckets(
            &mut self,
            request: impl tonic::IntoRequest<super::ListBucketsRequest>,
        ) -> Result<tonic::Response<super::ListBucketsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/ListBuckets",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Gets a bucket."]
        pub async fn get_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBucketRequest>,
        ) -> Result<tonic::Response<super::LogBucket>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/GetBucket",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Creates a bucket that can be used to store log entries. Once a bucket has"]
        #[doc = " been created, the region cannot be changed."]
        pub async fn create_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateBucketRequest>,
        ) -> Result<tonic::Response<super::LogBucket>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/CreateBucket",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Updates a bucket. This method replaces the following fields in the"]
        #[doc = " existing bucket with values from the new bucket: `retention_period`"]
        #[doc = ""]
        #[doc = " If the retention period is decreased and the bucket is locked,"]
        #[doc = " FAILED_PRECONDITION will be returned."]
        #[doc = ""]
        #[doc = " If the bucket has a LifecycleState of DELETE_REQUESTED, FAILED_PRECONDITION"]
        #[doc = " will be returned."]
        #[doc = ""]
        #[doc = " A buckets region may not be modified after it is created."]
        pub async fn update_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateBucketRequest>,
        ) -> Result<tonic::Response<super::LogBucket>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/UpdateBucket",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Deletes a bucket."]
        #[doc = " Moves the bucket to the DELETE_REQUESTED state. After 7 days, the"]
        #[doc = " bucket will be purged and all logs in the bucket will be permanently"]
        #[doc = " deleted."]
        pub async fn delete_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteBucketRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/DeleteBucket",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Undeletes a bucket. A bucket that has been deleted may be undeleted within"]
        #[doc = " the grace period of 7 days."]
        pub async fn undelete_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::UndeleteBucketRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/UndeleteBucket",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Lists views on a bucket."]
        pub async fn list_views(
            &mut self,
            request: impl tonic::IntoRequest<super::ListViewsRequest>,
        ) -> Result<tonic::Response<super::ListViewsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/ListViews",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Gets a view."]
        pub async fn get_view(
            &mut self,
            request: impl tonic::IntoRequest<super::GetViewRequest>,
        ) -> Result<tonic::Response<super::LogView>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.logging.v2.ConfigServiceV2/GetView");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Creates a view over logs in a bucket. A bucket may contain a maximum of"]
        #[doc = " 50 views."]
        pub async fn create_view(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateViewRequest>,
        ) -> Result<tonic::Response<super::LogView>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/CreateView",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Updates a view. This method replaces the following fields in the existing"]
        #[doc = " view with values from the new view: `filter`."]
        pub async fn update_view(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateViewRequest>,
        ) -> Result<tonic::Response<super::LogView>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/UpdateView",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Deletes a view from a bucket."]
        pub async fn delete_view(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteViewRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/DeleteView",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Lists sinks."]
        pub async fn list_sinks(
            &mut self,
            request: impl tonic::IntoRequest<super::ListSinksRequest>,
        ) -> Result<tonic::Response<super::ListSinksResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/ListSinks",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Gets a sink."]
        pub async fn get_sink(
            &mut self,
            request: impl tonic::IntoRequest<super::GetSinkRequest>,
        ) -> Result<tonic::Response<super::LogSink>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.logging.v2.ConfigServiceV2/GetSink");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Creates a sink that exports specified log entries to a destination. The"]
        #[doc = " export of newly-ingested log entries begins immediately, unless the sink's"]
        #[doc = " `writer_identity` is not permitted to write to the destination. A sink can"]
        #[doc = " export log entries only from the resource owning the sink."]
        pub async fn create_sink(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSinkRequest>,
        ) -> Result<tonic::Response<super::LogSink>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/CreateSink",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Updates a sink. This method replaces the following fields in the existing"]
        #[doc = " sink with values from the new sink: `destination`, and `filter`."]
        #[doc = ""]
        #[doc = " The updated sink might also have a new `writer_identity`; see the"]
        #[doc = " `unique_writer_identity` field."]
        pub async fn update_sink(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateSinkRequest>,
        ) -> Result<tonic::Response<super::LogSink>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/UpdateSink",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Deletes a sink. If the sink has a unique `writer_identity`, then that"]
        #[doc = " service account is also deleted."]
        pub async fn delete_sink(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteSinkRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/DeleteSink",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Lists all the exclusions in a parent resource."]
        pub async fn list_exclusions(
            &mut self,
            request: impl tonic::IntoRequest<super::ListExclusionsRequest>,
        ) -> Result<tonic::Response<super::ListExclusionsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/ListExclusions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Gets the description of an exclusion."]
        pub async fn get_exclusion(
            &mut self,
            request: impl tonic::IntoRequest<super::GetExclusionRequest>,
        ) -> Result<tonic::Response<super::LogExclusion>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/GetExclusion",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Creates a new exclusion in a specified parent resource."]
        #[doc = " Only log entries belonging to that resource can be excluded."]
        #[doc = " You can have up to 10 exclusions in a resource."]
        pub async fn create_exclusion(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateExclusionRequest>,
        ) -> Result<tonic::Response<super::LogExclusion>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/CreateExclusion",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Changes one or more properties of an existing exclusion."]
        pub async fn update_exclusion(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateExclusionRequest>,
        ) -> Result<tonic::Response<super::LogExclusion>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/UpdateExclusion",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Deletes an exclusion."]
        pub async fn delete_exclusion(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteExclusionRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/DeleteExclusion",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Gets the Logs Router CMEK settings for the given resource."]
        #[doc = ""]
        #[doc = " Note: CMEK for the Logs Router can currently only be configured for GCP"]
        #[doc = " organizations. Once configured, it applies to all projects and folders in"]
        #[doc = " the GCP organization."]
        #[doc = ""]
        #[doc = " See [Enabling CMEK for Logs"]
        #[doc = " Router](https://cloud.google.com/logging/docs/routing/managed-encryption)"]
        #[doc = " for more information."]
        pub async fn get_cmek_settings(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCmekSettingsRequest>,
        ) -> Result<tonic::Response<super::CmekSettings>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/GetCmekSettings",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Updates the Logs Router CMEK settings for the given resource."]
        #[doc = ""]
        #[doc = " Note: CMEK for the Logs Router can currently only be configured for GCP"]
        #[doc = " organizations. Once configured, it applies to all projects and folders in"]
        #[doc = " the GCP organization."]
        #[doc = ""]
        #[doc = " [UpdateCmekSettings][google.logging.v2.ConfigServiceV2.UpdateCmekSettings]"]
        #[doc = " will fail if 1) `kms_key_name` is invalid, or 2) the associated service"]
        #[doc = " account does not have the required"]
        #[doc = " `roles/cloudkms.cryptoKeyEncrypterDecrypter` role assigned for the key, or"]
        #[doc = " 3) access to the key is disabled."]
        #[doc = ""]
        #[doc = " See [Enabling CMEK for Logs"]
        #[doc = " Router](https://cloud.google.com/logging/docs/routing/managed-encryption)"]
        #[doc = " for more information."]
        pub async fn update_cmek_settings(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateCmekSettingsRequest>,
        ) -> Result<tonic::Response<super::CmekSettings>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.ConfigServiceV2/UpdateCmekSettings",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// The parameters to DeleteLog.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteLogRequest {
    /// Required. The resource name of the log to delete:
    ///
    ///     "projects/\[PROJECT_ID]/logs/[LOG_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/logs/[LOG_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/logs/[LOG_ID\]"
    ///     "folders/\[FOLDER_ID]/logs/[LOG_ID\]"
    ///
    /// `\[LOG_ID\]` must be URL-encoded. For example,
    /// `"projects/my-project-id/logs/syslog"`,
    /// `"organizations/1234567890/logs/cloudresourcemanager.googleapis.com%2Factivity"`.
    /// For more information about log names, see
    /// \[LogEntry][google.logging.v2.LogEntry\].
    #[prost(string, tag = "1")]
    pub log_name: ::prost::alloc::string::String,
}
/// The parameters to WriteLogEntries.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteLogEntriesRequest {
    /// Optional. A default log resource name that is assigned to all log entries
    /// in `entries` that do not specify a value for `log_name`:
    ///
    ///     "projects/\[PROJECT_ID]/logs/[LOG_ID\]"
    ///     "organizations/\[ORGANIZATION_ID]/logs/[LOG_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/logs/[LOG_ID\]"
    ///     "folders/\[FOLDER_ID]/logs/[LOG_ID\]"
    ///
    /// `\[LOG_ID\]` must be URL-encoded. For example:
    ///
    ///     "projects/my-project-id/logs/syslog"
    ///     "organizations/1234567890/logs/cloudresourcemanager.googleapis.com%2Factivity"
    ///
    /// The permission `logging.logEntries.create` is needed on each project,
    /// organization, billing account, or folder that is receiving new log
    /// entries, whether the resource is specified in `logName` or in an
    /// individual log entry.
    #[prost(string, tag = "1")]
    pub log_name: ::prost::alloc::string::String,
    /// Optional. A default monitored resource object that is assigned to all log
    /// entries in `entries` that do not specify a value for `resource`. Example:
    ///
    ///     { "type": "gce_instance",
    ///       "labels": {
    ///         "zone": "us-central1-a", "instance_id": "00000000000000000000" }}
    ///
    /// See \[LogEntry][google.logging.v2.LogEntry\].
    #[prost(message, optional, tag = "2")]
    pub resource: ::core::option::Option<super::super::api::MonitoredResource>,
    /// Optional. Default labels that are added to the `labels` field of all log
    /// entries in `entries`. If a log entry already has a label with the same key
    /// as a label in this parameter, then the log entry's label is not changed.
    /// See \[LogEntry][google.logging.v2.LogEntry\].
    #[prost(map = "string, string", tag = "3")]
    pub labels:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Required. The log entries to send to Logging. The order of log
    /// entries in this list does not matter. Values supplied in this method's
    /// `log_name`, `resource`, and `labels` fields are copied into those log
    /// entries in this list that do not include values for their corresponding
    /// fields. For more information, see the
    /// \[LogEntry][google.logging.v2.LogEntry\] type.
    ///
    /// If the `timestamp` or `insert_id` fields are missing in log entries, then
    /// this method supplies the current time or a unique identifier, respectively.
    /// The supplied values are chosen so that, among the log entries that did not
    /// supply their own values, the entries earlier in the list will sort before
    /// the entries later in the list. See the `entries.list` method.
    ///
    /// Log entries with timestamps that are more than the
    /// [logs retention period](<https://cloud.google.com/logging/quota-policy>) in
    /// the past or more than 24 hours in the future will not be available when
    /// calling `entries.list`. However, those log entries can still be [exported
    /// with
    /// LogSinks](<https://cloud.google.com/logging/docs/api/tasks/exporting-logs>).
    ///
    /// To improve throughput and to avoid exceeding the
    /// [quota limit](<https://cloud.google.com/logging/quota-policy>) for calls to
    /// `entries.write`, you should try to include several log entries in this
    /// list, rather than calling this method for each individual log entry.
    #[prost(message, repeated, tag = "4")]
    pub entries: ::prost::alloc::vec::Vec<LogEntry>,
    /// Optional. Whether valid entries should be written even if some other
    /// entries fail due to INVALID_ARGUMENT or PERMISSION_DENIED errors. If any
    /// entry is not written, then the response status is the error associated
    /// with one of the failed entries and the response includes error details
    /// keyed by the entries' zero-based index in the `entries.write` method.
    #[prost(bool, tag = "5")]
    pub partial_success: bool,
    /// Optional. If true, the request should expect normal response, but the
    /// entries won't be persisted nor exported. Useful for checking whether the
    /// logging API endpoints are working properly before sending valuable data.
    #[prost(bool, tag = "6")]
    pub dry_run: bool,
}
/// Result returned from WriteLogEntries.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteLogEntriesResponse {}
/// Error details for WriteLogEntries with partial success.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteLogEntriesPartialErrors {
    /// When `WriteLogEntriesRequest.partial_success` is true, records the error
    /// status for entries that were not written due to a permanent error, keyed
    /// by the entry's zero-based index in `WriteLogEntriesRequest.entries`.
    ///
    /// Failed requests for which no entries are written will not include
    /// per-entry errors.
    #[prost(map = "int32, message", tag = "1")]
    pub log_entry_errors: ::std::collections::HashMap<i32, super::super::rpc::Status>,
}
/// The parameters to `ListLogEntries`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListLogEntriesRequest {
    /// Required. Names of one or more parent resources from which to
    /// retrieve log entries:
    ///
    ///     "projects/\[PROJECT_ID\]"
    ///     "organizations/\[ORGANIZATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]"
    ///     "folders/\[FOLDER_ID\]"
    ///
    /// May alternatively be one or more views
    ///   projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]
    ///   organization/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]
    ///   billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]
    ///   folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]
    ///
    /// Projects listed in the `project_ids` field are added to this list.
    #[prost(string, repeated, tag = "8")]
    pub resource_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Optional. A filter that chooses which log entries to return.  See [Advanced
    /// Logs Queries](<https://cloud.google.com/logging/docs/view/advanced-queries>).
    /// Only log entries that match the filter are returned.  An empty filter
    /// matches all log entries in the resources listed in `resource_names`.
    /// Referencing a parent resource that is not listed in `resource_names` will
    /// cause the filter to return no results. The maximum length of the filter is
    /// 20000 characters.
    #[prost(string, tag = "2")]
    pub filter: ::prost::alloc::string::String,
    /// Optional. How the results should be sorted.  Presently, the only permitted
    /// values are `"timestamp asc"` (default) and `"timestamp desc"`. The first
    /// option returns entries in order of increasing values of
    /// `LogEntry.timestamp` (oldest first), and the second option returns entries
    /// in order of decreasing timestamps (newest first).  Entries with equal
    /// timestamps are returned in order of their `insert_id` values.
    #[prost(string, tag = "3")]
    pub order_by: ::prost::alloc::string::String,
    /// Optional. The maximum number of results to return from this request.
    /// Default is 50. If the value is negative or exceeds 1000,
    /// the request is rejected. The presence of `next_page_token` in the
    /// response indicates that more results might be available.
    #[prost(int32, tag = "4")]
    pub page_size: i32,
    /// Optional. If present, then retrieve the next batch of results from the
    /// preceding call to this method.  `page_token` must be the value of
    /// `next_page_token` from the previous response.  The values of other method
    /// parameters should be identical to those in the previous call.
    #[prost(string, tag = "5")]
    pub page_token: ::prost::alloc::string::String,
}
/// Result returned from `ListLogEntries`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListLogEntriesResponse {
    /// A list of log entries.  If `entries` is empty, `nextPageToken` may still be
    /// returned, indicating that more entries may exist.  See `nextPageToken` for
    /// more information.
    #[prost(message, repeated, tag = "1")]
    pub entries: ::prost::alloc::vec::Vec<LogEntry>,
    /// If there might be more results than those appearing in this response, then
    /// `nextPageToken` is included.  To get the next set of results, call this
    /// method again using the value of `nextPageToken` as `pageToken`.
    ///
    /// If a value for `next_page_token` appears and the `entries` field is empty,
    /// it means that the search found no log entries so far but it did not have
    /// time to search all the possible log entries.  Retry the method with this
    /// value for `page_token` to continue the search.  Alternatively, consider
    /// speeding up the search by changing your filter to specify a single log name
    /// or resource type, or to narrow the time range of the search.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The parameters to ListMonitoredResourceDescriptors
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMonitoredResourceDescriptorsRequest {
    /// Optional. The maximum number of results to return from this request.
    /// Non-positive values are ignored.  The presence of `nextPageToken` in the
    /// response indicates that more results might be available.
    #[prost(int32, tag = "1")]
    pub page_size: i32,
    /// Optional. If present, then retrieve the next batch of results from the
    /// preceding call to this method.  `pageToken` must be the value of
    /// `nextPageToken` from the previous response.  The values of other method
    /// parameters should be identical to those in the previous call.
    #[prost(string, tag = "2")]
    pub page_token: ::prost::alloc::string::String,
}
/// Result returned from ListMonitoredResourceDescriptors.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListMonitoredResourceDescriptorsResponse {
    /// A list of resource descriptors.
    #[prost(message, repeated, tag = "1")]
    pub resource_descriptors:
        ::prost::alloc::vec::Vec<super::super::api::MonitoredResourceDescriptor>,
    /// If there might be more results than those appearing in this response, then
    /// `nextPageToken` is included.  To get the next set of results, call this
    /// method again using the value of `nextPageToken` as `pageToken`.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The parameters to ListLogs.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListLogsRequest {
    /// Required. The resource name that owns the logs:
    ///
    ///     "projects/\[PROJECT_ID\]"
    ///     "organizations/\[ORGANIZATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]"
    ///     "folders/\[FOLDER_ID\]"
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Optional. The maximum number of results to return from this request.
    /// Non-positive values are ignored.  The presence of `nextPageToken` in the
    /// response indicates that more results might be available.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// Optional. If present, then retrieve the next batch of results from the
    /// preceding call to this method.  `pageToken` must be the value of
    /// `nextPageToken` from the previous response.  The values of other method
    /// parameters should be identical to those in the previous call.
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
    /// Optional. The resource name that owns the logs:
    ///   projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]
    ///   organization/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]
    ///   billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]
    ///   folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]
    ///
    /// To support legacy queries, it could also be:
    ///     "projects/\[PROJECT_ID\]"
    ///     "organizations/\[ORGANIZATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]"
    ///     "folders/\[FOLDER_ID\]"
    #[prost(string, repeated, tag = "8")]
    pub resource_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Result returned from ListLogs.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListLogsResponse {
    /// A list of log names. For example,
    /// `"projects/my-project/logs/syslog"` or
    /// `"organizations/123/logs/cloudresourcemanager.googleapis.com%2Factivity"`.
    #[prost(string, repeated, tag = "3")]
    pub log_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// If there might be more results than those appearing in this response, then
    /// `nextPageToken` is included.  To get the next set of results, call this
    /// method again using the value of `nextPageToken` as `pageToken`.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The parameters to `TailLogEntries`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TailLogEntriesRequest {
    /// Required. Name of a parent resource from which to retrieve log entries:
    ///
    ///     "projects/\[PROJECT_ID\]"
    ///     "organizations/\[ORGANIZATION_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID\]"
    ///     "folders/\[FOLDER_ID\]"
    ///
    /// May alternatively be one or more views:
    ///     "projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]"
    ///     "organization/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]"
    ///     "billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]"
    ///     "folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]"
    #[prost(string, repeated, tag = "1")]
    pub resource_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Optional. A filter that chooses which log entries to return.  See [Advanced
    /// Logs Filters](<https://cloud.google.com/logging/docs/view/advanced_filters>).
    /// Only log entries that match the filter are returned.  An empty filter
    /// matches all log entries in the resources listed in `resource_names`.
    /// Referencing a parent resource that is not in `resource_names` will cause
    /// the filter to return no results. The maximum length of the filter is 20000
    /// characters.
    #[prost(string, tag = "2")]
    pub filter: ::prost::alloc::string::String,
    /// Optional. The amount of time to buffer log entries at the server before
    /// being returned to prevent out of order results due to late arriving log
    /// entries. Valid values are between 0-60000 milliseconds. Defaults to 2000
    /// milliseconds.
    #[prost(message, optional, tag = "3")]
    pub buffer_window: ::core::option::Option<::prost_types::Duration>,
}
/// Result returned from `TailLogEntries`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TailLogEntriesResponse {
    /// A list of log entries. Each response in the stream will order entries with
    /// increasing values of `LogEntry.timestamp`. Ordering is not guaranteed
    /// between separate responses.
    #[prost(message, repeated, tag = "1")]
    pub entries: ::prost::alloc::vec::Vec<LogEntry>,
    /// If entries that otherwise would have been included in the session were not
    /// sent back to the client, counts of relevant entries omitted from the
    /// session with the reason that they were not included. There will be at most
    /// one of each reason per response. The counts represent the number of
    /// suppressed entries since the last streamed response.
    #[prost(message, repeated, tag = "2")]
    pub suppression_info: ::prost::alloc::vec::Vec<tail_log_entries_response::SuppressionInfo>,
}
/// Nested message and enum types in `TailLogEntriesResponse`.
pub mod tail_log_entries_response {
    /// Information about entries that were omitted from the session.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SuppressionInfo {
        /// The reason that entries were omitted from the session.
        #[prost(enumeration = "suppression_info::Reason", tag = "1")]
        pub reason: i32,
        /// A lower bound on the count of entries omitted due to `reason`.
        #[prost(int32, tag = "2")]
        pub suppressed_count: i32,
    }
    /// Nested message and enum types in `SuppressionInfo`.
    pub mod suppression_info {
        /// An indicator of why entries were omitted.
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
        )]
        #[repr(i32)]
        pub enum Reason {
            /// Unexpected default.
            Unspecified = 0,
            /// Indicates suppression occurred due to relevant entries being
            /// received in excess of rate limits. For quotas and limits, see
            /// [Logging API quotas and
            /// limits](<https://cloud.google.com/logging/quotas#api-limits>).
            RateLimit = 1,
            /// Indicates suppression occurred due to the client not consuming
            /// responses quickly enough.
            NotConsumed = 2,
        }
    }
}
#[doc = r" Generated client implementations."]
pub mod logging_service_v2_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = " Service for ingesting and querying logs."]
    #[derive(Debug, Clone)]
    pub struct LoggingServiceV2Client<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl LoggingServiceV2Client<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> LoggingServiceV2Client<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> LoggingServiceV2Client<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            LoggingServiceV2Client::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        #[doc = " Deletes all the log entries in a log. The log reappears if it receives new"]
        #[doc = " entries. Log entries written shortly before the delete operation might not"]
        #[doc = " be deleted. Entries received after the delete operation with a timestamp"]
        #[doc = " before the operation will be deleted."]
        pub async fn delete_log(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteLogRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.LoggingServiceV2/DeleteLog",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Writes log entries to Logging. This API method is the"]
        #[doc = " only way to send log entries to Logging. This method"]
        #[doc = " is used, directly or indirectly, by the Logging agent"]
        #[doc = " (fluentd) and all logging libraries configured to use Logging."]
        #[doc = " A single request may contain log entries for a maximum of 1000"]
        #[doc = " different resources (projects, organizations, billing accounts or"]
        #[doc = " folders)"]
        pub async fn write_log_entries(
            &mut self,
            request: impl tonic::IntoRequest<super::WriteLogEntriesRequest>,
        ) -> Result<tonic::Response<super::WriteLogEntriesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.LoggingServiceV2/WriteLogEntries",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Lists log entries.  Use this method to retrieve log entries that originated"]
        #[doc = " from a project/folder/organization/billing account.  For ways to export log"]
        #[doc = " entries, see [Exporting"]
        #[doc = " Logs](https://cloud.google.com/logging/docs/export)."]
        pub async fn list_log_entries(
            &mut self,
            request: impl tonic::IntoRequest<super::ListLogEntriesRequest>,
        ) -> Result<tonic::Response<super::ListLogEntriesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.LoggingServiceV2/ListLogEntries",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Lists the descriptors for monitored resource types used by Logging."]
        pub async fn list_monitored_resource_descriptors(
            &mut self,
            request: impl tonic::IntoRequest<super::ListMonitoredResourceDescriptorsRequest>,
        ) -> Result<tonic::Response<super::ListMonitoredResourceDescriptorsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.LoggingServiceV2/ListMonitoredResourceDescriptors",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Lists the logs in projects, organizations, folders, or billing accounts."]
        #[doc = " Only logs that have entries are listed."]
        pub async fn list_logs(
            &mut self,
            request: impl tonic::IntoRequest<super::ListLogsRequest>,
        ) -> Result<tonic::Response<super::ListLogsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.LoggingServiceV2/ListLogs",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Streaming read of log entries as they are ingested. Until the stream is"]
        #[doc = " terminated, it will continue reading logs."]
        pub async fn tail_log_entries(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::TailLogEntriesRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::TailLogEntriesResponse>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.logging.v2.LoggingServiceV2/TailLogEntries",
            );
            self.inner
                .streaming(request.into_streaming_request(), path, codec)
                .await
        }
    }
}
