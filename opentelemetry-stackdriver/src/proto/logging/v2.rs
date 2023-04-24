/// An individual entry in a log.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogEntry {
    /// Required. The resource name of the log to which this log entry belongs:
    ///
    ///      "projects/\[PROJECT_ID]/logs/[LOG_ID\]"
    ///      "organizations/\[ORGANIZATION_ID]/logs/[LOG_ID\]"
    ///      "billingAccounts/\[BILLING_ACCOUNT_ID]/logs/[LOG_ID\]"
    ///      "folders/\[FOLDER_ID]/logs/[LOG_ID\]"
    ///
    /// A project number may be used in place of PROJECT_ID. The project number is
    /// translated to its corresponding PROJECT_ID internally and the `log_name`
    /// field will contain PROJECT_ID in queries and exports.
    ///
    /// `\[LOG_ID\]` must be URL-encoded within `log_name`. Example:
    /// `"organizations/1234567890/logs/cloudresourcemanager.googleapis.com%2Factivity"`.
    ///
    /// `\[LOG_ID\]` must be less than 512 characters long and can only include the
    /// following characters: upper and lower case alphanumeric characters,
    /// forward-slash, underscore, hyphen, and period.
    ///
    /// For backward compatibility, if `log_name` begins with a forward-slash, such
    /// as `/projects/...`, then the log entry is ingested as usual, but the
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
    /// assigns its own unique identifier in this field.
    ///
    /// In queries, the `insert_id` is also used to order log entries that have
    /// the same `log_name` and `timestamp` values.
    #[prost(string, tag = "4")]
    pub insert_id: ::prost::alloc::string::String,
    /// Optional. Information about the HTTP request associated with this log entry, if
    /// applicable.
    #[prost(message, optional, tag = "7")]
    pub http_request: ::core::option::Option<super::r#type::HttpRequest>,
    /// Optional. A map of key, value pairs that provides additional information about the
    /// log entry. The labels can be user-defined or system-defined.
    ///
    /// User-defined labels are arbitrary key, value pairs that you can use to
    /// classify logs.
    ///
    /// System-defined labels are defined by GCP services for platform logs.
    /// They have two components - a service namespace component and the
    /// attribute name. For example: `compute.googleapis.com/resource_name`.
    ///
    /// Cloud Logging truncates label keys that exceed 512 B and label
    /// values that exceed 64 KB upon their associated log entry being
    /// written. The truncation is indicated by an ellipsis at the
    /// end of the character string.
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
    /// Optional. Information indicating this LogEntry is part of a sequence of multiple log
    /// entries split from a single LogEntry.
    #[prost(message, optional, tag = "35")]
    pub split: ::core::option::Option<LogSplit>,
    /// The log entry payload, which can be one of multiple types.
    #[prost(oneof = "log_entry::Payload", tags = "2, 3, 6")]
    pub payload: ::core::option::Option<log_entry::Payload>,
}
/// Nested message and enum types in `LogEntry`.
pub mod log_entry {
    /// The log entry payload, which can be one of multiple types.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        /// The log entry payload, represented as a protocol buffer. Some Google
        /// Cloud Platform services use this field for their log entry payloads.
        ///
        /// The following protocol buffer types are supported; user-defined types
        /// are not supported:
        ///
        ///    "type.googleapis.com/google.cloud.audit.AuditLog"
        ///    "type.googleapis.com/google.appengine.logging.v1.RequestLog"
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
#[allow(clippy::derive_partial_eq_without_eq)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
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
/// Additional information used to correlate multiple log entries. Used when a
/// single LogEntry would exceed the Google Cloud Logging size limit and is
/// split across multiple log entries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogSplit {
    /// A globally unique identifier for all log entries in a sequence of split log
    /// entries. All log entries with the same |LogSplit.uid| are assumed to be
    /// part of the same sequence of split log entries.
    #[prost(string, tag = "1")]
    pub uid: ::prost::alloc::string::String,
    /// The index of this LogEntry in the sequence of split log entries. Log
    /// entries are given |index| values 0, 1, ..., n-1 for a sequence of n log
    /// entries.
    #[prost(int32, tag = "2")]
    pub index: i32,
    /// The total number of log entries that the original LogEntry was split into.
    #[prost(int32, tag = "3")]
    pub total_splits: i32,
}
/// The parameters to DeleteLog.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteLogRequest {
    /// Required. The resource name of the log to delete:
    ///
    /// * `projects/\[PROJECT_ID]/logs/[LOG_ID\]`
    /// * `organizations/\[ORGANIZATION_ID]/logs/[LOG_ID\]`
    /// * `billingAccounts/\[BILLING_ACCOUNT_ID]/logs/[LOG_ID\]`
    /// * `folders/\[FOLDER_ID]/logs/[LOG_ID\]`
    ///
    /// `\[LOG_ID\]` must be URL-encoded. For example,
    /// `"projects/my-project-id/logs/syslog"`,
    /// `"organizations/123/logs/cloudaudit.googleapis.com%2Factivity"`.
    ///
    /// For more information about log names, see
    /// \[LogEntry][google.logging.v2.LogEntry\].
    #[prost(string, tag = "1")]
    pub log_name: ::prost::alloc::string::String,
}
/// The parameters to WriteLogEntries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteLogEntriesRequest {
    /// Optional. A default log resource name that is assigned to all log entries
    /// in `entries` that do not specify a value for `log_name`:
    ///
    /// * `projects/\[PROJECT_ID]/logs/[LOG_ID\]`
    /// * `organizations/\[ORGANIZATION_ID]/logs/[LOG_ID\]`
    /// * `billingAccounts/\[BILLING_ACCOUNT_ID]/logs/[LOG_ID\]`
    /// * `folders/\[FOLDER_ID]/logs/[LOG_ID\]`
    ///
    /// `\[LOG_ID\]` must be URL-encoded. For example:
    ///
    ///      "projects/my-project-id/logs/syslog"
    ///      "organizations/123/logs/cloudaudit.googleapis.com%2Factivity"
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
    ///      { "type": "gce_instance",
    ///        "labels": {
    ///          "zone": "us-central1-a", "instance_id": "00000000000000000000" }}
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
    /// [logs retention period](<https://cloud.google.com/logging/quotas>) in
    /// the past or more than 24 hours in the future will not be available when
    /// calling `entries.list`. However, those log entries can still be [exported
    /// with
    /// LogSinks](<https://cloud.google.com/logging/docs/api/tasks/exporting-logs>).
    ///
    /// To improve throughput and to avoid exceeding the
    /// [quota limit](<https://cloud.google.com/logging/quotas>) for calls to
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteLogEntriesResponse {}
/// Error details for WriteLogEntries with partial success.
#[allow(clippy::derive_partial_eq_without_eq)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListLogEntriesRequest {
    /// Required. Names of one or more parent resources from which to
    /// retrieve log entries:
    ///
    /// *  `projects/\[PROJECT_ID\]`
    /// *  `organizations/\[ORGANIZATION_ID\]`
    /// *  `billingAccounts/\[BILLING_ACCOUNT_ID\]`
    /// *  `folders/\[FOLDER_ID\]`
    ///
    /// May alternatively be one or more views:
    ///
    ///   * `projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `organizations/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
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
    /// Optional. The maximum number of results to return from this request. Default is 50.
    /// If the value is negative or exceeds 1000, the request is rejected. The
    /// presence of `next_page_token` in the response indicates that more results
    /// might be available.
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
#[allow(clippy::derive_partial_eq_without_eq)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListLogsRequest {
    /// Required. The resource name that owns the logs:
    ///
    /// *  `projects/\[PROJECT_ID\]`
    /// *  `organizations/\[ORGANIZATION_ID\]`
    /// *  `billingAccounts/\[BILLING_ACCOUNT_ID\]`
    /// *  `folders/\[FOLDER_ID\]`
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
    ///
    ///   * `projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `organizations/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///
    /// To support legacy queries, it could also be:
    ///
    /// *  `projects/\[PROJECT_ID\]`
    /// *  `organizations/\[ORGANIZATION_ID\]`
    /// *  `billingAccounts/\[BILLING_ACCOUNT_ID\]`
    /// *  `folders/\[FOLDER_ID\]`
    #[prost(string, repeated, tag = "8")]
    pub resource_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Result returned from ListLogs.
#[allow(clippy::derive_partial_eq_without_eq)]
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TailLogEntriesRequest {
    /// Required. Name of a parent resource from which to retrieve log entries:
    ///
    /// *  `projects/\[PROJECT_ID\]`
    /// *  `organizations/\[ORGANIZATION_ID\]`
    /// *  `billingAccounts/\[BILLING_ACCOUNT_ID\]`
    /// *  `folders/\[FOLDER_ID\]`
    ///
    /// May alternatively be one or more views:
    ///
    ///   * `projects/\[PROJECT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `organizations/\[ORGANIZATION_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `billingAccounts/\[BILLING_ACCOUNT_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
    ///   * `folders/\[FOLDER_ID]/locations/[LOCATION_ID]/buckets/[BUCKET_ID]/views/[VIEW_ID\]`
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
#[allow(clippy::derive_partial_eq_without_eq)]
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
    #[allow(clippy::derive_partial_eq_without_eq)]
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
        impl Reason {
            /// String value of the enum field names used in the ProtoBuf definition.
            ///
            /// The values are not transformed in any way and thus are considered stable
            /// (if the ProtoBuf definition does not change) and safe for programmatic use.
            pub fn as_str_name(&self) -> &'static str {
                match self {
                    Reason::Unspecified => "REASON_UNSPECIFIED",
                    Reason::RateLimit => "RATE_LIMIT",
                    Reason::NotConsumed => "NOT_CONSUMED",
                }
            }
            /// Creates an enum from field names used in the ProtoBuf definition.
            pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
                match value {
                    "REASON_UNSPECIFIED" => Some(Self::Unspecified),
                    "RATE_LIMIT" => Some(Self::RateLimit),
                    "NOT_CONSUMED" => Some(Self::NotConsumed),
                    _ => None,
                }
            }
        }
    }
}
/// Generated client implementations.
pub mod logging_service_v2_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    /// Service for ingesting and querying logs.
    #[derive(Debug, Clone)]
    pub struct LoggingServiceV2Client<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl LoggingServiceV2Client<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> LoggingServiceV2Client<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> LoggingServiceV2Client<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
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
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Deletes all the log entries in a log for the _Default Log Bucket. The log
        /// reappears if it receives new entries. Log entries written shortly before
        /// the delete operation might not be deleted. Entries received after the
        /// delete operation with a timestamp before the operation will be deleted.
        pub async fn delete_log(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteLogRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
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
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "google.logging.v2.LoggingServiceV2",
                "DeleteLog",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Writes log entries to Logging. This API method is the
        /// only way to send log entries to Logging. This method
        /// is used, directly or indirectly, by the Logging agent
        /// (fluentd) and all logging libraries configured to use Logging.
        /// A single request may contain log entries for a maximum of 1000
        /// different resources (projects, organizations, billing accounts or
        /// folders)
        pub async fn write_log_entries(
            &mut self,
            request: impl tonic::IntoRequest<super::WriteLogEntriesRequest>,
        ) -> std::result::Result<tonic::Response<super::WriteLogEntriesResponse>, tonic::Status>
        {
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
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "google.logging.v2.LoggingServiceV2",
                "WriteLogEntries",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Lists log entries.  Use this method to retrieve log entries that originated
        /// from a project/folder/organization/billing account.  For ways to export log
        /// entries, see [Exporting
        /// Logs](https://cloud.google.com/logging/docs/export).
        pub async fn list_log_entries(
            &mut self,
            request: impl tonic::IntoRequest<super::ListLogEntriesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListLogEntriesResponse>, tonic::Status>
        {
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
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "google.logging.v2.LoggingServiceV2",
                "ListLogEntries",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Lists the descriptors for monitored resource types used by Logging.
        pub async fn list_monitored_resource_descriptors(
            &mut self,
            request: impl tonic::IntoRequest<super::ListMonitoredResourceDescriptorsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListMonitoredResourceDescriptorsResponse>,
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
                "/google.logging.v2.LoggingServiceV2/ListMonitoredResourceDescriptors",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "google.logging.v2.LoggingServiceV2",
                "ListMonitoredResourceDescriptors",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Lists the logs in projects, organizations, folders, or billing accounts.
        /// Only logs that have entries are listed.
        pub async fn list_logs(
            &mut self,
            request: impl tonic::IntoRequest<super::ListLogsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListLogsResponse>, tonic::Status> {
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
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "google.logging.v2.LoggingServiceV2",
                "ListLogs",
            ));
            self.inner.unary(req, path, codec).await
        }
        /// Streaming read of log entries as they are ingested. Until the stream is
        /// terminated, it will continue reading logs.
        pub async fn tail_log_entries(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::TailLogEntriesRequest>,
        ) -> std::result::Result<
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
            let mut req = request.into_streaming_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "google.logging.v2.LoggingServiceV2",
                "TailLogEntries",
            ));
            self.inner.streaming(req, path, codec).await
        }
    }
}
