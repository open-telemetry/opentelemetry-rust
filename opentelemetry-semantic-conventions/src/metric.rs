// DO NOT EDIT, this is an auto-generated file
//
// If you want to update the file:
// - Edit the template at scripts/templates/registry/rust/metric.rs.j2
// - Run the script at scripts/generate-consts-from-spec.sh

//! # Metric Semantic Conventions
//!
//! The [metric semantic conventions] define a set of standardized attributes to
//! be used in `Meter`s.
//!
//! [metric semantic conventions]: https://github.com/open-telemetry/semantic-conventions/tree/main/model/metric
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::{global, KeyValue};
//! use opentelemetry_semantic_conventions as semconv;
//!
//! // Assumes we already have an initialized `MeterProvider`
//! // See: https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/metrics-basic/src/main.rs
//! // for an example
//! let meter = global::meter("mylibraryname");
//! let histogram = meter
//!     .u64_histogram(semconv::metric::HTTP_SERVER_REQUEST_DURATION)
//!     .with_unit("By")
//!     .with_description("Duration of HTTP server requests.")
//!     .build();
//! ```

/// ## Description
///
/// Number of exceptions caught by exception handling middleware.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Diagnostics`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{exception}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ASPNETCORE_DIAGNOSTICS_EXCEPTION_RESULT`] | `Required`
/// | [`crate::attribute::ASPNETCORE_DIAGNOSTICS_HANDLER_TYPE`] | `Conditionally_required`: if and only if the exception was handled by this handler.
/// | [`crate::attribute::ERROR_TYPE`] | `Required`
pub const ASPNETCORE_DIAGNOSTICS_EXCEPTIONS: &str = "aspnetcore.diagnostics.exceptions";

/// ## Description
///
/// Number of requests that are currently active on the server that hold a rate limiting lease.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.RateLimiting`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{request}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally_required`: if the matched endpoint for the request had a rate-limiting policy.
pub const ASPNETCORE_RATE_LIMITING_ACTIVE_REQUEST_LEASES: &str =
    "aspnetcore.rate_limiting.active_request_leases";

/// ## Description
///
/// Number of requests that are currently queued, waiting to acquire a rate limiting lease.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.RateLimiting`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{request}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally_required`: if the matched endpoint for the request had a rate-limiting policy.
pub const ASPNETCORE_RATE_LIMITING_QUEUED_REQUESTS: &str =
    "aspnetcore.rate_limiting.queued_requests";

/// ## Description
///
/// The time the request spent in a queue waiting to acquire a rate limiting lease.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.RateLimiting`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally_required`: if the matched endpoint for the request had a rate-limiting policy.
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_RESULT`] | `Required`
pub const ASPNETCORE_RATE_LIMITING_REQUEST_TIME_IN_QUEUE: &str =
    "aspnetcore.rate_limiting.request.time_in_queue";

/// ## Description
///
/// The duration of rate limiting lease held by requests on the server.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.RateLimiting`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally_required`: if the matched endpoint for the request had a rate-limiting policy.
pub const ASPNETCORE_RATE_LIMITING_REQUEST_LEASE_DURATION: &str =
    "aspnetcore.rate_limiting.request_lease.duration";

/// ## Description
///
/// Number of requests that tried to acquire a rate limiting lease.
///
/// ## Notes
///
/// Requests could be:
///
/// - Rejected by global or endpoint rate limiting policies
/// - Canceled while waiting for the lease.
///
/// Meter name: `Microsoft.AspNetCore.RateLimiting`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{request}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally_required`: if the matched endpoint for the request had a rate-limiting policy.
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_RESULT`] | `Required`
pub const ASPNETCORE_RATE_LIMITING_REQUESTS: &str = "aspnetcore.rate_limiting.requests";

/// ## Description
///
/// Number of requests that were attempted to be matched to an endpoint.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Routing`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{match_attempt}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ASPNETCORE_ROUTING_IS_FALLBACK`] | `Conditionally_required`: if and only if a route was successfully matched.
/// | [`crate::attribute::ASPNETCORE_ROUTING_MATCH_STATUS`] | `Required`
/// | [`crate::attribute::HTTP_ROUTE`] | `Conditionally_required`: if and only if a route was successfully matched.
pub const ASPNETCORE_ROUTING_MATCH_ATTEMPTS: &str = "aspnetcore.routing.match_attempts";

/// ## Description
///
/// Total CPU time consumed
///
/// ## Notes
///
/// Total CPU time consumed by the specific container on all available CPU cores
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::CPU_MODE`] | `Conditionally_required`: Required if mode is available, i.e. metrics coming from the Docker Stats API.
#[cfg(feature = "semconv_experimental")]
pub const CONTAINER_CPU_TIME: &str = "container.cpu.time";

/// ## Description
///
/// Container's CPU usage, measured in cpus. Range from 0 to the number of allocatable CPUs
///
/// ## Notes
///
/// CPU usage of the specific container on all available CPU cores, averaged over the sample window
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{cpu}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::CPU_MODE`] | `Conditionally_required`: Required if mode is available, i.e. metrics coming from the Docker Stats API.
#[cfg(feature = "semconv_experimental")]
pub const CONTAINER_CPU_USAGE: &str = "container.cpu.usage";

/// ## Description
///
/// Disk bytes for the container.
///
/// ## Notes
///
/// The total number of bytes read/written successfully (aggregated from all disks)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Recommended`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const CONTAINER_DISK_IO: &str = "container.disk.io";

/// ## Description
///
/// Memory usage of the container.
///
/// ## Notes
///
/// Memory usage of the container
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const CONTAINER_MEMORY_USAGE: &str = "container.memory.usage";

/// ## Description
///
/// Network bytes for the container.
///
/// ## Notes
///
/// The number of bytes sent/received on all network interfaces by the container
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const CONTAINER_NETWORK_IO: &str = "container.network.io";

/// ## Description
///
/// The time the container has been running
///
/// ## Notes
///
/// Instrumentations SHOULD use a gauge with type `double` and measure uptime in seconds as a floating point number with the highest precision available.
/// The actual accuracy would depend on the instrumentation and operating system
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const CONTAINER_UPTIME: &str = "container.uptime";

/// ## Description
///
/// The number of connections that are currently in state described by the `state` attribute
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
/// | [`crate::attribute::DB_CLIENT_CONNECTION_STATE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_COUNT: &str = "db.client.connection.count";

/// ## Description
///
/// The time it took to create a new connection
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_CREATE_TIME: &str = "db.client.connection.create_time";

/// ## Description
///
/// The maximum number of idle open connections allowed
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_IDLE_MAX: &str = "db.client.connection.idle.max";

/// ## Description
///
/// The minimum number of idle open connections allowed
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_IDLE_MIN: &str = "db.client.connection.idle.min";

/// ## Description
///
/// The maximum number of open connections allowed
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_MAX: &str = "db.client.connection.max";

/// ## Description
///
/// The number of current pending requests for an open connection
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{request}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_PENDING_REQUESTS: &str = "db.client.connection.pending_requests";

/// ## Description
///
/// The number of connection timeouts that have occurred trying to obtain a connection from the pool
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{timeout}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_TIMEOUTS: &str = "db.client.connection.timeouts";

/// ## Description
///
/// The time between borrowing a connection and returning it to the pool
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_USE_TIME: &str = "db.client.connection.use_time";

/// ## Description
///
/// The time it took to obtain an open connection from the pool
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTION_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_CONNECTION_WAIT_TIME: &str = "db.client.connection.wait_time";

/// ## Description
///
/// Deprecated, use `db.client.connection.create_time` instead. Note: the unit also changed from `ms` to `s`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `ms` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(
    note = "Replaced by `db.client.connection.create_time`. Note: the unit also changed from `ms` to `s`."
)]
pub const DB_CLIENT_CONNECTIONS_CREATE_TIME: &str = "db.client.connections.create_time";

/// ## Description
///
/// Deprecated, use `db.client.connection.idle.max` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `db.client.connection.idle.max`.")]
pub const DB_CLIENT_CONNECTIONS_IDLE_MAX: &str = "db.client.connections.idle.max";

/// ## Description
///
/// Deprecated, use `db.client.connection.idle.min` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `db.client.connection.idle.min`.")]
pub const DB_CLIENT_CONNECTIONS_IDLE_MIN: &str = "db.client.connections.idle.min";

/// ## Description
///
/// Deprecated, use `db.client.connection.max` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `db.client.connection.max`.")]
pub const DB_CLIENT_CONNECTIONS_MAX: &str = "db.client.connections.max";

/// ## Description
///
/// Deprecated, use `db.client.connection.pending_requests` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{request}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `db.client.connection.pending_requests`.")]
pub const DB_CLIENT_CONNECTIONS_PENDING_REQUESTS: &str = "db.client.connections.pending_requests";

/// ## Description
///
/// Deprecated, use `db.client.connection.timeouts` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{timeout}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `db.client.connection.timeouts`.")]
pub const DB_CLIENT_CONNECTIONS_TIMEOUTS: &str = "db.client.connections.timeouts";

/// ## Description
///
/// Deprecated, use `db.client.connection.count` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_STATE`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `db.client.connection.count`.")]
pub const DB_CLIENT_CONNECTIONS_USAGE: &str = "db.client.connections.usage";

/// ## Description
///
/// Deprecated, use `db.client.connection.use_time` instead. Note: the unit also changed from `ms` to `s`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `ms` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(
    note = "Replaced by `db.client.connection.use_time`. Note: the unit also changed from `ms` to `s`."
)]
pub const DB_CLIENT_CONNECTIONS_USE_TIME: &str = "db.client.connections.use_time";

/// ## Description
///
/// Deprecated, use `db.client.connection.wait_time` instead. Note: the unit also changed from `ms` to `s`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `ms` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
#[deprecated(
    note = "Replaced by `db.client.connection.wait_time`. Note: the unit also changed from `ms` to `s`."
)]
pub const DB_CLIENT_CONNECTIONS_WAIT_TIME: &str = "db.client.connections.wait_time";

/// ## Description
///
/// Number of active client instances
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{instance}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If using a port other than the default port for this DBMS and if `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_COSMOSDB_ACTIVE_INSTANCE_COUNT: &str =
    "db.client.cosmosdb.active_instance.count";

/// ## Description
///
/// [Request charge](https://learn.microsoft.com/azure/cosmos-db/request-units) consumed by the operation
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `{request_unit}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_COLLECTION_NAME`] | `Conditionally_required`: If available.
/// | [`crate::attribute::DB_COSMOSDB_CONSISTENCY_LEVEL`] | `Conditionally_required`: If available.
/// | [`crate::attribute::DB_COSMOSDB_REGIONS_CONTACTED`] | `{"recommended": "if available"}`
/// | [`crate::attribute::DB_COSMOSDB_SUB_STATUS_CODE`] | `Conditionally_required`: when response was received and contained sub-code.
/// | [`crate::attribute::DB_NAMESPACE`] | `Conditionally_required`: If available.
/// | [`crate::attribute::DB_OPERATION_NAME`] | `Conditionally_required`: If readily available and if there is a single operation name that describes the database call. The operation name MAY be parsed from the query text, in which case it SHOULD be the single operation name found in the query.
/// | [`crate::attribute::DB_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If the operation failed and status code is available.
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the operation failed.
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If using a port other than the default port for this DBMS and if `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_COSMOSDB_OPERATION_REQUEST_CHARGE: &str =
    "db.client.cosmosdb.operation.request_charge";

/// ## Description
///
/// Duration of database client operations.
///
/// ## Notes
///
/// Batch operations SHOULD be recorded as a single operation
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_COLLECTION_NAME`] | `Conditionally_required`: If readily available and if a database call is performed on a single collection. The collection name MAY be parsed from the query text, in which case it SHOULD be the single collection name in the query.
/// | [`crate::attribute::DB_NAMESPACE`] | `Conditionally_required`: If available.
/// | [`crate::attribute::DB_OPERATION_NAME`] | `Conditionally_required`: If readily available and if there is a single operation name that describes the database call. The operation name MAY be parsed from the query text, in which case it SHOULD be the single operation name found in the query.
/// | [`crate::attribute::DB_QUERY_SUMMARY`] | `{"recommended": "if readily available or if instrumentation supports query summarization."}`
/// | [`crate::attribute::DB_QUERY_TEXT`] | `Opt_in`
/// | [`crate::attribute::DB_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If the operation failed and status code is available.
/// | [`crate::attribute::DB_SYSTEM`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the operation failed.
/// | [`crate::attribute::NETWORK_PEER_ADDRESS`] | `{"recommended": "if applicable for this database system."}`
/// | [`crate::attribute::NETWORK_PEER_PORT`] | `{"recommended": "if and only if `network.peer.address` is set."}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If using a port other than the default port for this DBMS and if `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_OPERATION_DURATION: &str = "db.client.operation.duration";

/// ## Description
///
/// The actual number of records returned by the database operation
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `{row}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DB_COLLECTION_NAME`] | `Conditionally_required`: If readily available and if a database call is performed on a single collection. The collection name MAY be parsed from the query text, in which case it SHOULD be the single collection name in the query.
/// | [`crate::attribute::DB_NAMESPACE`] | `Conditionally_required`: If available.
/// | [`crate::attribute::DB_OPERATION_NAME`] | `Conditionally_required`: If readily available and if there is a single operation name that describes the database call. The operation name MAY be parsed from the query text, in which case it SHOULD be the single operation name found in the query.
/// | [`crate::attribute::DB_QUERY_SUMMARY`] | `{"recommended": "if readily available or if instrumentation supports query summarization."}`
/// | [`crate::attribute::DB_QUERY_TEXT`] | `Opt_in`
/// | [`crate::attribute::DB_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If the operation failed and status code is available.
/// | [`crate::attribute::DB_SYSTEM`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the operation failed.
/// | [`crate::attribute::NETWORK_PEER_ADDRESS`] | `{"recommended": "if applicable for this database system."}`
/// | [`crate::attribute::NETWORK_PEER_PORT`] | `{"recommended": "if and only if `network.peer.address` is set."}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If using a port other than the default port for this DBMS and if `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const DB_CLIENT_RESPONSE_RETURNED_ROWS: &str = "db.client.response.returned_rows";

/// ## Description
///
/// Measures the time taken to perform a DNS lookup
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DNS_QUESTION_NAME`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: if and only if an error has occurred.
#[cfg(feature = "semconv_experimental")]
pub const DNS_LOOKUP_DURATION: &str = "dns.lookup.duration";

/// ## Description
///
/// The number of .NET assemblies that are currently loaded.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`AppDomain.CurrentDomain.GetAssemblies().Length`](https://learn.microsoft.com/dotnet/api/system.appdomain.getassemblies)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{assembly}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_ASSEMBLY_COUNT: &str = "dotnet.assembly.count";

/// ## Description
///
/// The number of exceptions that have been thrown in managed code.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as counting calls to [`AppDomain.CurrentDomain.FirstChanceException`](https://learn.microsoft.com/dotnet/api/system.appdomain.firstchanceexception)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{exception}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_EXCEPTIONS: &str = "dotnet.exceptions";

/// ## Description
///
/// The number of garbage collections that have occurred since the process has started.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric uses the [`GC.CollectionCount(int generation)`](https://learn.microsoft.com/dotnet/api/system.gc.collectioncount) API to calculate exclusive collections per generation
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{collection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DOTNET_GC_HEAP_GENERATION`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_GC_COLLECTIONS: &str = "dotnet.gc.collections";

/// ## Description
///
/// The *approximate* number of bytes allocated on the managed GC heap since the process has started. The returned value does not include any native allocations.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`GC.GetTotalAllocatedBytes()`](https://learn.microsoft.com/dotnet/api/system.gc.gettotalallocatedbytes)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_GC_HEAP_TOTAL_ALLOCATED: &str = "dotnet.gc.heap.total_allocated";

/// ## Description
///
/// The heap fragmentation, as observed during the latest garbage collection.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`GC.GetGCMemoryInfo().GenerationInfo.FragmentationAfterBytes`](https://learn.microsoft.com/dotnet/api/system.gcgenerationinfo.fragmentationafterbytes)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DOTNET_GC_HEAP_GENERATION`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_GC_LAST_COLLECTION_HEAP_FRAGMENTATION_SIZE: &str =
    "dotnet.gc.last_collection.heap.fragmentation.size";

/// ## Description
///
/// The managed GC heap size (including fragmentation), as observed during the latest garbage collection.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`GC.GetGCMemoryInfo().GenerationInfo.SizeAfterBytes`](https://learn.microsoft.com/dotnet/api/system.gcgenerationinfo.sizeafterbytes)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DOTNET_GC_HEAP_GENERATION`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_GC_LAST_COLLECTION_HEAP_SIZE: &str = "dotnet.gc.last_collection.heap.size";

/// ## Description
///
/// The amount of committed virtual memory in use by the .NET GC, as observed during the latest garbage collection.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`GC.GetGCMemoryInfo().TotalCommittedBytes`](https://learn.microsoft.com/dotnet/api/system.gcmemoryinfo.totalcommittedbytes). Committed virtual memory may be larger than the heap size because it includes both memory for storing existing objects (the heap size) and some extra memory that is ready to handle newly allocated objects in the future
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_GC_LAST_COLLECTION_MEMORY_COMMITTED_SIZE: &str =
    "dotnet.gc.last_collection.memory.committed_size";

/// ## Description
///
/// The total amount of time paused in GC since the process has started.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`GC.GetTotalPauseDuration()`](https://learn.microsoft.com/dotnet/api/system.gc.gettotalpauseduration)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_GC_PAUSE_TIME: &str = "dotnet.gc.pause.time";

/// ## Description
///
/// The amount of time the JIT compiler has spent compiling methods since the process has started.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`JitInfo.GetCompilationTime()`](https://learn.microsoft.com/dotnet/api/system.runtime.jitinfo.getcompilationtime)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_JIT_COMPILATION_TIME: &str = "dotnet.jit.compilation.time";

/// ## Description
///
/// Count of bytes of intermediate language that have been compiled since the process has started.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`JitInfo.GetCompiledILBytes()`](https://learn.microsoft.com/dotnet/api/system.runtime.jitinfo.getcompiledilbytes)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_JIT_COMPILED_IL_SIZE: &str = "dotnet.jit.compiled_il.size";

/// ## Description
///
/// The number of times the JIT compiler (re)compiled methods since the process has started.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`JitInfo.GetCompiledMethodCount()`](https://learn.microsoft.com/dotnet/api/system.runtime.jitinfo.getcompiledmethodcount)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{method}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_JIT_COMPILED_METHODS: &str = "dotnet.jit.compiled_methods";

/// ## Description
///
/// The number of times there was contention when trying to acquire a monitor lock since the process has started.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`Monitor.LockContentionCount`](https://learn.microsoft.com/dotnet/api/system.threading.monitor.lockcontentioncount)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{contention}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_MONITOR_LOCK_CONTENTIONS: &str = "dotnet.monitor.lock_contentions";

/// ## Description
///
/// The number of processors available to the process.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as accessing [`Environment.ProcessorCount`](https://learn.microsoft.com/dotnet/api/system.environment.processorcount)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{cpu}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_PROCESS_CPU_COUNT: &str = "dotnet.process.cpu.count";

/// ## Description
///
/// CPU time used by the process.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as accessing the corresponding processor time properties on [`System.Diagnostics.Process`](https://learn.microsoft.com/dotnet/api/system.diagnostics.process)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::CPU_MODE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_PROCESS_CPU_TIME: &str = "dotnet.process.cpu.time";

/// ## Description
///
/// The number of bytes of physical memory mapped to the process context.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`Environment.WorkingSet`](https://learn.microsoft.com/dotnet/api/system.environment.workingset)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_PROCESS_MEMORY_WORKING_SET: &str = "dotnet.process.memory.working_set";

/// ## Description
///
/// The number of work items that are currently queued to be processed by the thread pool.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`ThreadPool.PendingWorkItemCount`](https://learn.microsoft.com/dotnet/api/system.threading.threadpool.pendingworkitemcount)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{work_item}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_THREAD_POOL_QUEUE_LENGTH: &str = "dotnet.thread_pool.queue.length";

/// ## Description
///
/// The number of thread pool threads that currently exist.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`ThreadPool.ThreadCount`](https://learn.microsoft.com/dotnet/api/system.threading.threadpool.threadcount)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{thread}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_THREAD_POOL_THREAD_COUNT: &str = "dotnet.thread_pool.thread.count";

/// ## Description
///
/// The number of work items that the thread pool has completed since the process has started.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`ThreadPool.CompletedWorkItemCount`](https://learn.microsoft.com/dotnet/api/system.threading.threadpool.completedworkitemcount)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{work_item}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_THREAD_POOL_WORK_ITEM_COUNT: &str = "dotnet.thread_pool.work_item.count";

/// ## Description
///
/// The number of timer instances that are currently active.
///
/// ## Notes
///
/// Meter name: `System.Runtime`; Added in: .NET 9.0.
/// This metric reports the same values as calling [`Timer.ActiveCount`](https://learn.microsoft.com/dotnet/api/system.threading.timer.activecount)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{timer}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const DOTNET_TIMER_COUNT: &str = "dotnet.timer.count";

/// ## Description
///
/// Number of invocation cold starts
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{coldstart}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_COLDSTARTS: &str = "faas.coldstarts";

/// ## Description
///
/// Distribution of CPU usage per invocation
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_CPU_USAGE: &str = "faas.cpu_usage";

/// ## Description
///
/// Number of invocation errors
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{error}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_ERRORS: &str = "faas.errors";

/// ## Description
///
/// Measures the duration of the function's initialization, such as a cold start
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_INIT_DURATION: &str = "faas.init_duration";

/// ## Description
///
/// Number of successful invocations
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{invocation}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_INVOCATIONS: &str = "faas.invocations";

/// ## Description
///
/// Measures the duration of the function's logic execution
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_INVOKE_DURATION: &str = "faas.invoke_duration";

/// ## Description
///
/// Distribution of max memory usage per invocation
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_MEM_USAGE: &str = "faas.mem_usage";

/// ## Description
///
/// Distribution of net I/O usage per invocation
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_NET_IO: &str = "faas.net_io";

/// ## Description
///
/// Number of invocation timeouts
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{timeout}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::FAAS_TRIGGER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const FAAS_TIMEOUTS: &str = "faas.timeouts";

/// ## Description
///
/// GenAI operation duration
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: if the operation ended in an error
/// | [`crate::attribute::GEN_AI_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::GEN_AI_REQUEST_MODEL`] | `Conditionally_required`: If available.
/// | [`crate::attribute::GEN_AI_RESPONSE_MODEL`] | `Recommended`
/// | [`crate::attribute::GEN_AI_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const GEN_AI_CLIENT_OPERATION_DURATION: &str = "gen_ai.client.operation.duration";

/// ## Description
///
/// Measures number of input and output tokens used
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `{token}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::GEN_AI_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::GEN_AI_REQUEST_MODEL`] | `Conditionally_required`: If available.
/// | [`crate::attribute::GEN_AI_RESPONSE_MODEL`] | `Recommended`
/// | [`crate::attribute::GEN_AI_SYSTEM`] | `Required`
/// | [`crate::attribute::GEN_AI_TOKEN_TYPE`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const GEN_AI_CLIENT_TOKEN_USAGE: &str = "gen_ai.client.token.usage";

/// ## Description
///
/// Generative AI server request duration such as time-to-last byte or last output token
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: if the operation ended in an error
/// | [`crate::attribute::GEN_AI_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::GEN_AI_REQUEST_MODEL`] | `Conditionally_required`: If available.
/// | [`crate::attribute::GEN_AI_RESPONSE_MODEL`] | `Recommended`
/// | [`crate::attribute::GEN_AI_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const GEN_AI_SERVER_REQUEST_DURATION: &str = "gen_ai.server.request.duration";

/// ## Description
///
/// Time per output token generated after the first token for successful responses
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::GEN_AI_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::GEN_AI_REQUEST_MODEL`] | `Conditionally_required`: If available.
/// | [`crate::attribute::GEN_AI_RESPONSE_MODEL`] | `Recommended`
/// | [`crate::attribute::GEN_AI_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const GEN_AI_SERVER_TIME_PER_OUTPUT_TOKEN: &str = "gen_ai.server.time_per_output_token";

/// ## Description
///
/// Time to generate first token for successful responses
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::GEN_AI_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::GEN_AI_REQUEST_MODEL`] | `Conditionally_required`: If available.
/// | [`crate::attribute::GEN_AI_RESPONSE_MODEL`] | `Recommended`
/// | [`crate::attribute::GEN_AI_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally_required`: If `server.address` is set.
#[cfg(feature = "semconv_experimental")]
pub const GEN_AI_SERVER_TIME_TO_FIRST_TOKEN: &str = "gen_ai.server.time_to_first_token";

/// ## Description
///
/// Heap size target percentage configured by the user, otherwise 100.
///
/// ## Notes
///
/// The value range is \\[0.0,100.0\\]. Computed from `/gc/gogc:percent`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `%` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const GO_CONFIG_GOGC: &str = "go.config.gogc";

/// ## Description
///
/// Count of live goroutines.
///
/// ## Notes
///
/// Computed from `/sched/goroutines:goroutines`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{goroutine}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const GO_GOROUTINE_COUNT: &str = "go.goroutine.count";

/// ## Description
///
/// Memory allocated to the heap by the application.
///
/// ## Notes
///
/// Computed from `/gc/heap/allocs:bytes`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const GO_MEMORY_ALLOCATED: &str = "go.memory.allocated";

/// ## Description
///
/// Count of allocations to the heap by the application.
///
/// ## Notes
///
/// Computed from `/gc/heap/allocs:objects`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{allocation}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const GO_MEMORY_ALLOCATIONS: &str = "go.memory.allocations";

/// ## Description
///
/// Heap size target for the end of the GC cycle.
///
/// ## Notes
///
/// Computed from `/gc/heap/goal:bytes`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const GO_MEMORY_GC_GOAL: &str = "go.memory.gc.goal";

/// ## Description
///
/// Go runtime memory limit configured by the user, if a limit exists.
///
/// ## Notes
///
/// Computed from `/gc/gomemlimit:bytes`. This metric is excluded if the limit obtained from the Go runtime is math.MaxInt64
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const GO_MEMORY_LIMIT: &str = "go.memory.limit";

/// ## Description
///
/// Memory used by the Go runtime.
///
/// ## Notes
///
/// Computed from `(/memory/classes/total:bytes - /memory/classes/heap/released:bytes)`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::GO_MEMORY_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const GO_MEMORY_USED: &str = "go.memory.used";

/// ## Description
///
/// The number of OS threads that can execute user-level Go code simultaneously.
///
/// ## Notes
///
/// Computed from `/sched/gomaxprocs:threads`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{thread}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const GO_PROCESSOR_LIMIT: &str = "go.processor.limit";

/// ## Description
///
/// The time goroutines have spent in the scheduler in a runnable state before actually running.
///
/// ## Notes
///
/// Computed from `/sched/latencies:seconds`. Bucket boundaries are provided by the runtime, and are subject to change
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const GO_SCHEDULE_DURATION: &str = "go.schedule.duration";

/// ## Description
///
/// Number of active HTTP requests
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{request}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Opt_in`
/// | [`crate::attribute::URL_TEMPLATE`] | `Conditionally_required`: If available.
#[cfg(feature = "semconv_experimental")]
pub const HTTP_CLIENT_ACTIVE_REQUESTS: &str = "http.client.active_requests";

/// ## Description
///
/// The duration of the successfully established outbound HTTP connections
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_PEER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Opt_in`
#[cfg(feature = "semconv_experimental")]
pub const HTTP_CLIENT_CONNECTION_DURATION: &str = "http.client.connection.duration";

/// ## Description
///
/// Number of outbound HTTP connections that are currently active or idle on the client
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::HTTP_CONNECTION_STATE`] | `Required`
/// | [`crate::attribute::NETWORK_PEER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Opt_in`
#[cfg(feature = "semconv_experimental")]
pub const HTTP_CLIENT_OPEN_CONNECTIONS: &str = "http.client.open_connections";

/// ## Description
///
/// Size of HTTP client request bodies.
///
/// ## Notes
///
/// The size of the request payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If and only if one was received/sent.
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally_required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Opt_in`
/// | [`crate::attribute::URL_TEMPLATE`] | `Conditionally_required`: If available.
#[cfg(feature = "semconv_experimental")]
pub const HTTP_CLIENT_REQUEST_BODY_SIZE: &str = "http.client.request.body.size";

/// ## Description
///
/// Duration of HTTP client requests
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If and only if one was received/sent.
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally_required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Opt_in`
/// | [`crate::attribute::URL_TEMPLATE`] | `Opt_in`
pub const HTTP_CLIENT_REQUEST_DURATION: &str = "http.client.request.duration";

/// ## Description
///
/// Size of HTTP client response bodies.
///
/// ## Notes
///
/// The size of the response payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If and only if one was received/sent.
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally_required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Opt_in`
/// | [`crate::attribute::URL_TEMPLATE`] | `Conditionally_required`: If available.
#[cfg(feature = "semconv_experimental")]
pub const HTTP_CLIENT_RESPONSE_BODY_SIZE: &str = "http.client.response.body.size";

/// ## Description
///
/// Number of active HTTP server requests
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{request}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Opt_in`
/// | [`crate::attribute::SERVER_PORT`] | `Opt_in`
/// | [`crate::attribute::URL_SCHEME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const HTTP_SERVER_ACTIVE_REQUESTS: &str = "http.server.active_requests";

/// ## Description
///
/// Size of HTTP server request bodies.
///
/// ## Notes
///
/// The size of the request payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If and only if one was received/sent.
/// | [`crate::attribute::HTTP_ROUTE`] | `Conditionally_required`: If and only if it's available
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally_required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Opt_in`
/// | [`crate::attribute::SERVER_PORT`] | `Opt_in`
/// | [`crate::attribute::URL_SCHEME`] | `Required`
/// | [`crate::attribute::USER_AGENT_SYNTHETIC_TYPE`] | `Opt_in`
#[cfg(feature = "semconv_experimental")]
pub const HTTP_SERVER_REQUEST_BODY_SIZE: &str = "http.server.request.body.size";

/// ## Description
///
/// Duration of HTTP server requests
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If and only if one was received/sent.
/// | [`crate::attribute::HTTP_ROUTE`] | `Conditionally_required`: If and only if it's available
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally_required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Opt_in`
/// | [`crate::attribute::SERVER_PORT`] | `Opt_in`
/// | [`crate::attribute::URL_SCHEME`] | `Required`
/// | [`crate::attribute::USER_AGENT_SYNTHETIC_TYPE`] | `Opt_in`
pub const HTTP_SERVER_REQUEST_DURATION: &str = "http.server.request.duration";

/// ## Description
///
/// Size of HTTP server response bodies.
///
/// ## Notes
///
/// The size of the response payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally_required`: If and only if one was received/sent.
/// | [`crate::attribute::HTTP_ROUTE`] | `Conditionally_required`: If and only if it's available
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally_required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Opt_in`
/// | [`crate::attribute::SERVER_PORT`] | `Opt_in`
/// | [`crate::attribute::URL_SCHEME`] | `Required`
/// | [`crate::attribute::USER_AGENT_SYNTHETIC_TYPE`] | `Opt_in`
#[cfg(feature = "semconv_experimental")]
pub const HTTP_SERVER_RESPONSE_BODY_SIZE: &str = "http.server.response.body.size";

/// ## Description
///
/// Energy consumed by the component
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `J` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::HW_ID`] | `Required`
/// | [`crate::attribute::HW_NAME`] | `Recommended`
/// | [`crate::attribute::HW_PARENT`] | `Recommended`
/// | [`crate::attribute::HW_TYPE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const HW_ENERGY: &str = "hw.energy";

/// ## Description
///
/// Number of errors encountered by the component
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{error}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: if and only if an error has occurred
/// | [`crate::attribute::HW_ID`] | `Required`
/// | [`crate::attribute::HW_NAME`] | `Recommended`
/// | [`crate::attribute::HW_PARENT`] | `Recommended`
/// | [`crate::attribute::HW_TYPE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const HW_ERRORS: &str = "hw.errors";

/// ## Description
///
/// Instantaneous power consumed by the component
///
/// ## Notes
///
/// It is recommended to report `hw.energy` instead of `hw.power` when possible
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `W` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::HW_ID`] | `Required`
/// | [`crate::attribute::HW_NAME`] | `Recommended`
/// | [`crate::attribute::HW_PARENT`] | `Recommended`
/// | [`crate::attribute::HW_TYPE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const HW_POWER: &str = "hw.power";

/// ## Description
///
/// Operational status: `1` (true) or `0` (false) for each of the possible states
///
/// ## Notes
///
/// `hw.status` is currently specified as an *UpDownCounter* but would ideally be represented using a [*StateSet* as defined in OpenMetrics](https://github.com/OpenObservability/OpenMetrics/blob/main/specification/OpenMetrics.md#stateset). This semantic convention will be updated once *StateSet* is specified in OpenTelemetry. This planned change is not expected to have any consequence on the way users query their timeseries backend to retrieve the values of `hw.status` over time
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::HW_ID`] | `Required`
/// | [`crate::attribute::HW_NAME`] | `Recommended`
/// | [`crate::attribute::HW_PARENT`] | `Recommended`
/// | [`crate::attribute::HW_STATE`] | `Required`
/// | [`crate::attribute::HW_TYPE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const HW_STATUS: &str = "hw.status";

/// ## Description
///
/// Number of buffers in the pool
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{buffer}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_BUFFER_POOL_NAME`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const JVM_BUFFER_COUNT: &str = "jvm.buffer.count";

/// ## Description
///
/// Measure of total memory capacity of buffers
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_BUFFER_POOL_NAME`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const JVM_BUFFER_MEMORY_LIMIT: &str = "jvm.buffer.memory.limit";

/// ## Description
///
/// Deprecated, use `jvm.buffer.memory.used` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_BUFFER_POOL_NAME`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `jvm.buffer.memory.used`.")]
pub const JVM_BUFFER_MEMORY_USAGE: &str = "jvm.buffer.memory.usage";

/// ## Description
///
/// Measure of memory used by buffers
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_BUFFER_POOL_NAME`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const JVM_BUFFER_MEMORY_USED: &str = "jvm.buffer.memory.used";

/// ## Description
///
/// Number of classes currently loaded
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{class}` |
/// | Status: | `Stable`  |
pub const JVM_CLASS_COUNT: &str = "jvm.class.count";

/// ## Description
///
/// Number of classes loaded since JVM start
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{class}` |
/// | Status: | `Stable`  |
pub const JVM_CLASS_LOADED: &str = "jvm.class.loaded";

/// ## Description
///
/// Number of classes unloaded since JVM start
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{class}` |
/// | Status: | `Stable`  |
pub const JVM_CLASS_UNLOADED: &str = "jvm.class.unloaded";

/// ## Description
///
/// Number of processors available to the Java virtual machine
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{cpu}` |
/// | Status: | `Stable`  |
pub const JVM_CPU_COUNT: &str = "jvm.cpu.count";

/// ## Description
///
/// Recent CPU utilization for the process as reported by the JVM.
///
/// ## Notes
///
/// The value range is \\[0.0,1.0\\]. This utilization is not defined as being for the specific interval since last measurement (unlike `system.cpu.utilization`). [Reference](https://docs.oracle.com/en/java/javase/17/docs/api/jdk.management/com/sun/management/OperatingSystemMXBean.html#getProcessCpuLoad())
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Stable`  |
pub const JVM_CPU_RECENT_UTILIZATION: &str = "jvm.cpu.recent_utilization";

/// ## Description
///
/// CPU time used by the process as reported by the JVM
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
pub const JVM_CPU_TIME: &str = "jvm.cpu.time";

/// ## Description
///
/// Duration of JVM garbage collection actions
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_GC_ACTION`] | `Recommended`
/// | [`crate::attribute::JVM_GC_NAME`] | `Recommended`
pub const JVM_GC_DURATION: &str = "jvm.gc.duration";

/// ## Description
///
/// Measure of memory committed
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_MEMORY_POOL_NAME`] | `Recommended`
/// | [`crate::attribute::JVM_MEMORY_TYPE`] | `Recommended`
pub const JVM_MEMORY_COMMITTED: &str = "jvm.memory.committed";

/// ## Description
///
/// Measure of initial memory requested
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_MEMORY_POOL_NAME`] | `Recommended`
/// | [`crate::attribute::JVM_MEMORY_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const JVM_MEMORY_INIT: &str = "jvm.memory.init";

/// ## Description
///
/// Measure of max obtainable memory
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_MEMORY_POOL_NAME`] | `Recommended`
/// | [`crate::attribute::JVM_MEMORY_TYPE`] | `Recommended`
pub const JVM_MEMORY_LIMIT: &str = "jvm.memory.limit";

/// ## Description
///
/// Measure of memory used
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_MEMORY_POOL_NAME`] | `Recommended`
/// | [`crate::attribute::JVM_MEMORY_TYPE`] | `Recommended`
pub const JVM_MEMORY_USED: &str = "jvm.memory.used";

/// ## Description
///
/// Measure of memory used, as measured after the most recent garbage collection event on this pool
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_MEMORY_POOL_NAME`] | `Recommended`
/// | [`crate::attribute::JVM_MEMORY_TYPE`] | `Recommended`
pub const JVM_MEMORY_USED_AFTER_LAST_GC: &str = "jvm.memory.used_after_last_gc";

/// ## Description
///
/// Average CPU load of the whole system for the last minute as reported by the JVM.
///
/// ## Notes
///
/// The value range is \\[0,n\\], where n is the number of CPU cores - or a negative number if the value is not available. This utilization is not defined as being for the specific interval since last measurement (unlike `system.cpu.utilization`). [Reference](https://docs.oracle.com/en/java/javase/17/docs/api/java.management/java/lang/management/OperatingSystemMXBean.html#getSystemLoadAverage())
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{run_queue_item}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const JVM_SYSTEM_CPU_LOAD_1M: &str = "jvm.system.cpu.load_1m";

/// ## Description
///
/// Recent CPU utilization for the whole system as reported by the JVM.
///
/// ## Notes
///
/// The value range is \\[0.0,1.0\\]. This utilization is not defined as being for the specific interval since last measurement (unlike `system.cpu.utilization`). [Reference](https://docs.oracle.com/en/java/javase/17/docs/api/jdk.management/com/sun/management/OperatingSystemMXBean.html#getCpuLoad())
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const JVM_SYSTEM_CPU_UTILIZATION: &str = "jvm.system.cpu.utilization";

/// ## Description
///
/// Number of executing platform threads
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{thread}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::JVM_THREAD_DAEMON`] | `Recommended`
/// | [`crate::attribute::JVM_THREAD_STATE`] | `Recommended`
pub const JVM_THREAD_COUNT: &str = "jvm.thread.count";

/// ## Description
///
/// Total CPU time consumed
///
/// ## Notes
///
/// Total CPU time consumed by the specific Node on all available CPU cores
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const K8S_NODE_CPU_TIME: &str = "k8s.node.cpu.time";

/// ## Description
///
/// Node's CPU usage, measured in cpus. Range from 0 to the number of allocatable CPUs
///
/// ## Notes
///
/// CPU usage of the specific Node on all available CPU cores, averaged over the sample window
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{cpu}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const K8S_NODE_CPU_USAGE: &str = "k8s.node.cpu.usage";

/// ## Description
///
/// Memory usage of the Node
///
/// ## Notes
///
/// Total memory usage of the Node
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const K8S_NODE_MEMORY_USAGE: &str = "k8s.node.memory.usage";

/// ## Description
///
/// Node network errors
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{error}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const K8S_NODE_NETWORK_ERRORS: &str = "k8s.node.network.errors";

/// ## Description
///
/// Network bytes for the Node
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const K8S_NODE_NETWORK_IO: &str = "k8s.node.network.io";

/// ## Description
///
/// The time the Node has been running
///
/// ## Notes
///
/// Instrumentations SHOULD use a gauge with type `double` and measure uptime in seconds as a floating point number with the highest precision available.
/// The actual accuracy would depend on the instrumentation and operating system
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const K8S_NODE_UPTIME: &str = "k8s.node.uptime";

/// ## Description
///
/// Total CPU time consumed
///
/// ## Notes
///
/// Total CPU time consumed by the specific Pod on all available CPU cores
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const K8S_POD_CPU_TIME: &str = "k8s.pod.cpu.time";

/// ## Description
///
/// Pod's CPU usage, measured in cpus. Range from 0 to the number of allocatable CPUs
///
/// ## Notes
///
/// CPU usage of the specific Pod on all available CPU cores, averaged over the sample window
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{cpu}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const K8S_POD_CPU_USAGE: &str = "k8s.pod.cpu.usage";

/// ## Description
///
/// Memory usage of the Pod
///
/// ## Notes
///
/// Total memory usage of the Pod
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const K8S_POD_MEMORY_USAGE: &str = "k8s.pod.memory.usage";

/// ## Description
///
/// Pod network errors
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{error}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const K8S_POD_NETWORK_ERRORS: &str = "k8s.pod.network.errors";

/// ## Description
///
/// Network bytes for the Pod
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const K8S_POD_NETWORK_IO: &str = "k8s.pod.network.io";

/// ## Description
///
/// The time the Pod has been running
///
/// ## Notes
///
/// Instrumentations SHOULD use a gauge with type `double` and measure uptime in seconds as a floating point number with the highest precision available.
/// The actual accuracy would depend on the instrumentation and operating system
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const K8S_POD_UPTIME: &str = "k8s.pod.uptime";

/// ## Description
///
/// Number of connections that are currently active on the server.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Server.Kestrel`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::NETWORK_TYPE`] | `{"recommended": "if the transport is `tcp` or `udp`"}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
pub const KESTREL_ACTIVE_CONNECTIONS: &str = "kestrel.active_connections";

/// ## Description
///
/// Number of TLS handshakes that are currently in progress on the server.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Server.Kestrel`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{handshake}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::NETWORK_TYPE`] | `{"recommended": "if the transport is `tcp` or `udp`"}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
pub const KESTREL_ACTIVE_TLS_HANDSHAKES: &str = "kestrel.active_tls_handshakes";

/// ## Description
///
/// The duration of connections on the server.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Server.Kestrel`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: if and only if an error has occurred.
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::NETWORK_TYPE`] | `{"recommended": "if the transport is `tcp` or `udp`"}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
/// | [`crate::attribute::TLS_PROTOCOL_VERSION`] | `Recommended`
pub const KESTREL_CONNECTION_DURATION: &str = "kestrel.connection.duration";

/// ## Description
///
/// Number of connections that are currently queued and are waiting to start.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Server.Kestrel`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::NETWORK_TYPE`] | `{"recommended": "if the transport is `tcp` or `udp`"}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
pub const KESTREL_QUEUED_CONNECTIONS: &str = "kestrel.queued_connections";

/// ## Description
///
/// Number of HTTP requests on multiplexed connections (HTTP/2 and HTTP/3) that are currently queued and are waiting to start.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Server.Kestrel`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{request}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::NETWORK_TYPE`] | `{"recommended": "if the transport is `tcp` or `udp`"}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
pub const KESTREL_QUEUED_REQUESTS: &str = "kestrel.queued_requests";

/// ## Description
///
/// Number of connections rejected by the server.
///
/// ## Notes
///
/// Connections are rejected when the currently active count exceeds the value configured with `MaxConcurrentConnections`.
/// Meter name: `Microsoft.AspNetCore.Server.Kestrel`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{connection}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::NETWORK_TYPE`] | `{"recommended": "if the transport is `tcp` or `udp`"}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
pub const KESTREL_REJECTED_CONNECTIONS: &str = "kestrel.rejected_connections";

/// ## Description
///
/// The duration of TLS handshakes on the server.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Server.Kestrel`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: if and only if an error has occurred.
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::NETWORK_TYPE`] | `{"recommended": "if the transport is `tcp` or `udp`"}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
/// | [`crate::attribute::TLS_PROTOCOL_VERSION`] | `Recommended`
pub const KESTREL_TLS_HANDSHAKE_DURATION: &str = "kestrel.tls_handshake.duration";

/// ## Description
///
/// Number of connections that are currently upgraded (WebSockets). .
///
/// ## Notes
///
/// The counter only tracks HTTP/1.1 connections.
///
/// Meter name: `Microsoft.AspNetCore.Server.Kestrel`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::NETWORK_TYPE`] | `{"recommended": "if the transport is `tcp` or `udp`"}`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
pub const KESTREL_UPGRADED_CONNECTIONS: &str = "kestrel.upgraded_connections";

/// ## Description
///
/// Number of messages that were delivered to the application.
///
/// ## Notes
///
/// Records the number of messages pulled from the broker or number of messages dispatched to the application in push-based scenarios.
/// The metric SHOULD be reported once per message delivery. For example, if receiving and processing operations are both instrumented for a single message delivery, this counter is incremented when the message is received and not reported when it is processed
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{message}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_CONSUMER_GROUP_NAME`] | `Conditionally_required`: if applicable.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally_required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Recommended`
/// | [`crate::attribute::MESSAGING_DESTINATION_SUBSCRIPTION_NAME`] | `Conditionally_required`: if applicable.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally_required`: if available.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const MESSAGING_CLIENT_CONSUMED_MESSAGES: &str = "messaging.client.consumed.messages";

/// ## Description
///
/// Duration of messaging operation initiated by a producer or consumer client.
///
/// ## Notes
///
/// This metric SHOULD NOT be used to report processing duration - processing duration is reported in `messaging.process.duration` metric
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_CONSUMER_GROUP_NAME`] | `Conditionally_required`: if applicable.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally_required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Recommended`
/// | [`crate::attribute::MESSAGING_DESTINATION_SUBSCRIPTION_NAME`] | `Conditionally_required`: if applicable.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally_required`: if available.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::MESSAGING_OPERATION_TYPE`] | `Conditionally_required`: If applicable.
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const MESSAGING_CLIENT_OPERATION_DURATION: &str = "messaging.client.operation.duration";

/// ## Description
///
/// Deprecated. Use `messaging.client.sent.messages` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{message}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally_required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Recommended`
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally_required`: if available.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `messaging.client.sent.messages`.")]
pub const MESSAGING_CLIENT_PUBLISHED_MESSAGES: &str = "messaging.client.published.messages";

/// ## Description
///
/// Number of messages producer attempted to send to the broker.
///
/// ## Notes
///
/// This metric MUST NOT count messages that were created but haven't yet been sent
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{message}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally_required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Recommended`
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally_required`: if available.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const MESSAGING_CLIENT_SENT_MESSAGES: &str = "messaging.client.sent.messages";

/// ## Description
///
/// Duration of processing operation.
///
/// ## Notes
///
/// This metric MUST be reported for operations with `messaging.operation.type` that matches `process`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_CONSUMER_GROUP_NAME`] | `Conditionally_required`: if applicable.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally_required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Recommended`
/// | [`crate::attribute::MESSAGING_DESTINATION_SUBSCRIPTION_NAME`] | `Conditionally_required`: if applicable.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally_required`: if available.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const MESSAGING_PROCESS_DURATION: &str = "messaging.process.duration";

/// ## Description
///
/// Deprecated. Use `messaging.client.consumed.messages` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{message}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `messaging.client.consumed.messages`.")]
pub const MESSAGING_PROCESS_MESSAGES: &str = "messaging.process.messages";

/// ## Description
///
/// Deprecated. Use `messaging.client.operation.duration` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `messaging.client.operation.duration`.")]
pub const MESSAGING_PUBLISH_DURATION: &str = "messaging.publish.duration";

/// ## Description
///
/// Deprecated. Use `messaging.client.produced.messages` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{message}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `messaging.client.produced.messages`.")]
pub const MESSAGING_PUBLISH_MESSAGES: &str = "messaging.publish.messages";

/// ## Description
///
/// Deprecated. Use `messaging.client.operation.duration` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `messaging.client.operation.duration`.")]
pub const MESSAGING_RECEIVE_DURATION: &str = "messaging.receive.duration";

/// ## Description
///
/// Deprecated. Use `messaging.client.consumed.messages` instead
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{message}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally_required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_OPERATION_NAME`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally_required`: If available.
/// | [`crate::attribute::SERVER_PORT`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
#[deprecated(note = "Replaced by `messaging.client.consumed.messages`.")]
pub const MESSAGING_RECEIVE_MESSAGES: &str = "messaging.receive.messages";

/// ## Description
///
/// Event loop maximum delay.
///
/// ## Notes
///
/// Value can be retrieved from value `histogram.max` of [`perf_hooks.monitorEventLoopDelay([options])`](https://nodejs.org/api/perf_hooks.html#perf_hooksmonitoreventloopdelayoptions)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_DELAY_MAX: &str = "nodejs.eventloop.delay.max";

/// ## Description
///
/// Event loop mean delay.
///
/// ## Notes
///
/// Value can be retrieved from value `histogram.mean` of [`perf_hooks.monitorEventLoopDelay([options])`](https://nodejs.org/api/perf_hooks.html#perf_hooksmonitoreventloopdelayoptions)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_DELAY_MEAN: &str = "nodejs.eventloop.delay.mean";

/// ## Description
///
/// Event loop minimum delay.
///
/// ## Notes
///
/// Value can be retrieved from value `histogram.min` of [`perf_hooks.monitorEventLoopDelay([options])`](https://nodejs.org/api/perf_hooks.html#perf_hooksmonitoreventloopdelayoptions)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_DELAY_MIN: &str = "nodejs.eventloop.delay.min";

/// ## Description
///
/// Event loop 50 percentile delay.
///
/// ## Notes
///
/// Value can be retrieved from value `histogram.percentile(50)` of [`perf_hooks.monitorEventLoopDelay([options])`](https://nodejs.org/api/perf_hooks.html#perf_hooksmonitoreventloopdelayoptions)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_DELAY_P50: &str = "nodejs.eventloop.delay.p50";

/// ## Description
///
/// Event loop 90 percentile delay.
///
/// ## Notes
///
/// Value can be retrieved from value `histogram.percentile(90)` of [`perf_hooks.monitorEventLoopDelay([options])`](https://nodejs.org/api/perf_hooks.html#perf_hooksmonitoreventloopdelayoptions)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_DELAY_P90: &str = "nodejs.eventloop.delay.p90";

/// ## Description
///
/// Event loop 99 percentile delay.
///
/// ## Notes
///
/// Value can be retrieved from value `histogram.percentile(99)` of [`perf_hooks.monitorEventLoopDelay([options])`](https://nodejs.org/api/perf_hooks.html#perf_hooksmonitoreventloopdelayoptions)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_DELAY_P99: &str = "nodejs.eventloop.delay.p99";

/// ## Description
///
/// Event loop standard deviation delay.
///
/// ## Notes
///
/// Value can be retrieved from value `histogram.stddev` of [`perf_hooks.monitorEventLoopDelay([options])`](https://nodejs.org/api/perf_hooks.html#perf_hooksmonitoreventloopdelayoptions)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_DELAY_STDDEV: &str = "nodejs.eventloop.delay.stddev";

/// ## Description
///
/// Cumulative duration of time the event loop has been in each state.
///
/// ## Notes
///
/// Value can be retrieved from [`performance.eventLoopUtilization([utilization1[, utilization2]])`](https://nodejs.org/api/perf_hooks.html#performanceeventlooputilizationutilization1-utilization2)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NODEJS_EVENTLOOP_STATE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_TIME: &str = "nodejs.eventloop.time";

/// ## Description
///
/// Event loop utilization.
///
/// ## Notes
///
/// The value range is \[0.0, 1.0\] and can be retrieved from [`performance.eventLoopUtilization([utilization1[, utilization2]])`](https://nodejs.org/api/perf_hooks.html#performanceeventlooputilizationutilization1-utilization2)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const NODEJS_EVENTLOOP_UTILIZATION: &str = "nodejs.eventloop.utilization";

/// ## Description
///
/// Number of times the process has been context switched
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{count}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::PROCESS_CONTEXT_SWITCH_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_CONTEXT_SWITCHES: &str = "process.context_switches";

/// ## Description
///
/// Total CPU seconds broken down by different states
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::CPU_MODE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_CPU_TIME: &str = "process.cpu.time";

/// ## Description
///
/// Difference in process.cpu.time since the last measurement, divided by the elapsed time and number of CPUs available to the process
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::CPU_MODE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_CPU_UTILIZATION: &str = "process.cpu.utilization";

/// ## Description
///
/// Disk bytes transferred
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_DISK_IO: &str = "process.disk.io";

/// ## Description
///
/// The amount of physical memory in use
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_MEMORY_USAGE: &str = "process.memory.usage";

/// ## Description
///
/// The amount of committed virtual memory
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_MEMORY_VIRTUAL: &str = "process.memory.virtual";

/// ## Description
///
/// Network bytes transferred
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_NETWORK_IO: &str = "process.network.io";

/// ## Description
///
/// Number of file descriptors in use by the process
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{count}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_OPEN_FILE_DESCRIPTOR_COUNT: &str = "process.open_file_descriptor.count";

/// ## Description
///
/// Number of page faults the process has made
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{fault}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::PROCESS_PAGING_FAULT_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_PAGING_FAULTS: &str = "process.paging.faults";

/// ## Description
///
/// Process threads count
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{thread}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_THREAD_COUNT: &str = "process.thread.count";

/// ## Description
///
/// The time the process has been running.
///
/// ## Notes
///
/// Instrumentations SHOULD use a gauge with type `double` and measure uptime in seconds as a floating point number with the highest precision available.
/// The actual accuracy would depend on the instrumentation and operating system
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const PROCESS_UPTIME: &str = "process.uptime";

/// ## Description
///
/// Measures the duration of outbound RPC.
///
/// ## Notes
///
/// While streaming RPCs may record this metric as start-of-batch
/// to end-of-batch, it's hard to interpret in practice.
///
/// **Streaming**: N/A
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `ms` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_CLIENT_DURATION: &str = "rpc.client.duration";

/// ## Description
///
/// Measures the size of RPC request messages (uncompressed).
///
/// ## Notes
///
/// **Streaming**: Recorded per message in a streaming batch
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_CLIENT_REQUEST_SIZE: &str = "rpc.client.request.size";

/// ## Description
///
/// Measures the number of messages received per RPC.
///
/// ## Notes
///
/// Should be 1 for all non-streaming RPCs.
///
/// **Streaming**: This metric is required for server and client streaming RPCs
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `{count}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_CLIENT_REQUESTS_PER_RPC: &str = "rpc.client.requests_per_rpc";

/// ## Description
///
/// Measures the size of RPC response messages (uncompressed).
///
/// ## Notes
///
/// **Streaming**: Recorded per response in a streaming batch
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_CLIENT_RESPONSE_SIZE: &str = "rpc.client.response.size";

/// ## Description
///
/// Measures the number of messages sent per RPC.
///
/// ## Notes
///
/// Should be 1 for all non-streaming RPCs.
///
/// **Streaming**: This metric is required for server and client streaming RPCs
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `{count}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_CLIENT_RESPONSES_PER_RPC: &str = "rpc.client.responses_per_rpc";

/// ## Description
///
/// Measures the duration of inbound RPC.
///
/// ## Notes
///
/// While streaming RPCs may record this metric as start-of-batch
/// to end-of-batch, it's hard to interpret in practice.
///
/// **Streaming**: N/A
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `ms` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_SERVER_DURATION: &str = "rpc.server.duration";

/// ## Description
///
/// Measures the size of RPC request messages (uncompressed).
///
/// ## Notes
///
/// **Streaming**: Recorded per message in a streaming batch
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_SERVER_REQUEST_SIZE: &str = "rpc.server.request.size";

/// ## Description
///
/// Measures the number of messages received per RPC.
///
/// ## Notes
///
/// Should be 1 for all non-streaming RPCs.
///
/// **Streaming** : This metric is required for server and client streaming RPCs
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `{count}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_SERVER_REQUESTS_PER_RPC: &str = "rpc.server.requests_per_rpc";

/// ## Description
///
/// Measures the size of RPC response messages (uncompressed).
///
/// ## Notes
///
/// **Streaming**: Recorded per response in a streaming batch
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_SERVER_RESPONSE_SIZE: &str = "rpc.server.response.size";

/// ## Description
///
/// Measures the number of messages sent per RPC.
///
/// ## Notes
///
/// Should be 1 for all non-streaming RPCs.
///
/// **Streaming**: This metric is required for server and client streaming RPCs
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `{count}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const RPC_SERVER_RESPONSES_PER_RPC: &str = "rpc.server.responses_per_rpc";

/// ## Description
///
/// Number of connections that are currently active on the server.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Http.Connections`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SIGNALR_CONNECTION_STATUS`] | `Recommended`
/// | [`crate::attribute::SIGNALR_TRANSPORT`] | `Recommended`
pub const SIGNALR_SERVER_ACTIVE_CONNECTIONS: &str = "signalr.server.active_connections";

/// ## Description
///
/// The duration of connections on the server.
///
/// ## Notes
///
/// Meter name: `Microsoft.AspNetCore.Http.Connections`; Added in: ASP.NET Core 8.0
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SIGNALR_CONNECTION_STATUS`] | `Recommended`
/// | [`crate::attribute::SIGNALR_TRANSPORT`] | `Recommended`
pub const SIGNALR_SERVER_CONNECTION_DURATION: &str = "signalr.server.connection.duration";

/// ## Description
///
/// Reports the current frequency of the CPU in Hz
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{Hz}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_CPU_LOGICAL_NUMBER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_CPU_FREQUENCY: &str = "system.cpu.frequency";

/// ## Description
///
/// Reports the number of logical (virtual) processor cores created by the operating system to manage multitasking
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{cpu}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_CPU_LOGICAL_COUNT: &str = "system.cpu.logical.count";

/// ## Description
///
/// Reports the number of actual physical processor cores on the hardware
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{cpu}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_CPU_PHYSICAL_COUNT: &str = "system.cpu.physical.count";

/// ## Description
///
/// Seconds each logical CPU spent on each mode
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::CPU_MODE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_CPU_LOGICAL_NUMBER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_CPU_TIME: &str = "system.cpu.time";

/// ## Description
///
/// Difference in system.cpu.time since the last measurement, divided by the elapsed time and number of logical CPUs
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::CPU_MODE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_CPU_LOGICAL_NUMBER`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_CPU_UTILIZATION: &str = "system.cpu.utilization";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Recommended`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_DISK_IO: &str = "system.disk.io";

/// ## Description
///
/// Time disk spent activated
///
/// ## Notes
///
/// The real elapsed time ("wall clock") used in the I/O path (time from operations running in parallel are not counted). Measured as:
///
/// - Linux: Field 13 from [procfs-diskstats](https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats)
/// - Windows: The complement of
///   ["Disk% Idle Time"](https://learn.microsoft.com/archive/blogs/askcore/windows-performance-monitor-disk-counters-explained#windows-performance-monitor-disk-counters-explained)
///   performance counter: `uptime * (100 - "Disk\% Idle Time") / 100`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_DISK_IO_TIME: &str = "system.disk.io_time";

/// ## Description
///
/// The total storage capacity of the disk
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_DISK_LIMIT: &str = "system.disk.limit";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{operation}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Recommended`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_DISK_MERGED: &str = "system.disk.merged";

/// ## Description
///
/// Sum of the time each operation took to complete
///
/// ## Notes
///
/// Because it is the sum of time each request took, parallel-issued requests each contribute to make the count grow. Measured as:
///
/// - Linux: Fields 7 & 11 from [procfs-diskstats](https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats)
/// - Windows: "Avg. Disk sec/Read" perf counter multiplied by "Disk Reads/sec" perf counter (similar for Writes)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Recommended`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_DISK_OPERATION_TIME: &str = "system.disk.operation_time";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{operation}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Recommended`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_DISK_OPERATIONS: &str = "system.disk.operations";

/// ## Description
///
/// The total storage capacity of the filesystem
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MODE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MOUNTPOINT`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_FILESYSTEM_LIMIT: &str = "system.filesystem.limit";

/// ## Description
///
/// Reports a filesystem's space usage across different states.
///
/// ## Notes
///
/// The sum of all `system.filesystem.usage` values over the different `system.filesystem.state` attributes
/// SHOULD equal the total storage capacity of the filesystem, that is `system.filesystem.limit`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MODE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MOUNTPOINT`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_STATE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_FILESYSTEM_USAGE: &str = "system.filesystem.usage";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MODE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MOUNTPOINT`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_STATE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_FILESYSTEM_UTILIZATION: &str = "system.filesystem.utilization";

/// ## Description
///
/// An estimate of how much memory is available for starting new applications, without causing swapping
///
/// ## Notes
///
/// This is an alternative to `system.memory.usage` metric with `state=free`.
/// Linux starting from 3.14 exports "available" memory. It takes "free" memory as a baseline, and then factors in kernel-specific values.
/// This is supposed to be more accurate than just "free" memory.
/// For reference, see the calculations [here](https://superuser.com/a/980821).
/// See also `MemAvailable` in [/proc/meminfo](https://man7.org/linux/man-pages/man5/proc.5.html)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_LINUX_MEMORY_AVAILABLE: &str = "system.linux.memory.available";

/// ## Description
///
/// Reports the memory used by the Linux kernel for managing caches of frequently used objects.
///
/// ## Notes
///
/// The sum over the `reclaimable` and `unreclaimable` state values in `linux.memory.slab.usage` SHOULD be equal to the total slab memory available on the system.
/// Note that the total slab memory is not constant and may vary over time.
/// See also the [Slab allocator](https://blogs.oracle.com/linux/post/understanding-linux-kernel-memory-statistics) and `Slab` in [/proc/meminfo](https://man7.org/linux/man-pages/man5/proc.5.html)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::LINUX_MEMORY_SLAB_STATE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_LINUX_MEMORY_SLAB_USAGE: &str = "system.linux.memory.slab.usage";

/// ## Description
///
/// Total memory available in the system.
///
/// ## Notes
///
/// Its value SHOULD equal the sum of `system.memory.state` over all states
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_MEMORY_LIMIT: &str = "system.memory.limit";

/// ## Description
///
/// Shared memory used (mostly by tmpfs).
///
/// ## Notes
///
/// Equivalent of `shared` from [`free` command](https://man7.org/linux/man-pages/man1/free.1.html) or
/// `Shmem` from [`/proc/meminfo`](https://man7.org/linux/man-pages/man5/proc.5.html)"
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_MEMORY_SHARED: &str = "system.memory.shared";

/// ## Description
///
/// Reports memory in use by state.
///
/// ## Notes
///
/// The sum over all `system.memory.state` values SHOULD equal the total memory
/// available on the system, that is `system.memory.limit`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_MEMORY_STATE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_MEMORY_USAGE: &str = "system.memory.usage";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_MEMORY_STATE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_MEMORY_UTILIZATION: &str = "system.memory.utilization";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{connection}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Recommended`
/// | [`crate::attribute::SYSTEM_NETWORK_STATE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_NETWORK_CONNECTIONS: &str = "system.network.connections";

/// ## Description
///
/// Count of packets that are dropped or discarded even though there was no error
///
/// ## Notes
///
/// Measured as:
///
/// - Linux: the `drop` column in `/proc/dev/net` ([source](https://web.archive.org/web/20180321091318/http://www.onlamp.com/pub/a/linux/2000/11/16/LinuxAdmin.html))
/// - Windows: [`InDiscards`/`OutDiscards`](https://docs.microsoft.com/windows/win32/api/netioapi/ns-netioapi-mib_if_row2)
///   from [`GetIfEntry2`](https://docs.microsoft.com/windows/win32/api/netioapi/nf-netioapi-getifentry2)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{packet}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_NETWORK_DROPPED: &str = "system.network.dropped";

/// ## Description
///
/// Count of network errors detected
///
/// ## Notes
///
/// Measured as:
///
/// - Linux: the `errs` column in `/proc/dev/net` ([source](https://web.archive.org/web/20180321091318/http://www.onlamp.com/pub/a/linux/2000/11/16/LinuxAdmin.html)).
/// - Windows: [`InErrors`/`OutErrors`](https://docs.microsoft.com/windows/win32/api/netioapi/ns-netioapi-mib_if_row2)
///   from [`GetIfEntry2`](https://docs.microsoft.com/windows/win32/api/netioapi/nf-netioapi-getifentry2)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{error}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_NETWORK_ERRORS: &str = "system.network.errors";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_INTERFACE_NAME`] | `Recommended`
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_NETWORK_IO: &str = "system.network.io";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{packet}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Recommended`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_NETWORK_PACKETS: &str = "system.network.packets";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{fault}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_PAGING_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_PAGING_FAULTS: &str = "system.paging.faults";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{operation}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_PAGING_DIRECTION`] | `Recommended`
/// | [`crate::attribute::SYSTEM_PAGING_TYPE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_PAGING_OPERATIONS: &str = "system.paging.operations";

/// ## Description
///
/// Unix swap or windows pagefile usage
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_PAGING_STATE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_PAGING_USAGE: &str = "system.paging.usage";

/// ## Description
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Recommended`
/// | [`crate::attribute::SYSTEM_PAGING_STATE`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_PAGING_UTILIZATION: &str = "system.paging.utilization";

/// ## Description
///
/// Total number of processes in each state
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{process}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::SYSTEM_PROCESS_STATUS`] | `Recommended`
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_PROCESS_COUNT: &str = "system.process.count";

/// ## Description
///
/// Total number of processes created over uptime of the host
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{process}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_PROCESS_CREATED: &str = "system.process.created";

/// ## Description
///
/// The time the system has been running
///
/// ## Notes
///
/// Instrumentations SHOULD use a gauge with type `double` and measure uptime in seconds as a floating point number with the highest precision available.
/// The actual accuracy would depend on the instrumentation and operating system
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const SYSTEM_UPTIME: &str = "system.uptime";

/// ## Description
///
/// Garbage collection duration.
///
/// ## Notes
///
/// The values can be retrieve from [`perf_hooks.PerformanceObserver(...).observe({ entryTypes: ['gc'] })`](https://nodejs.org/api/perf_hooks.html#performanceobserverobserveoptions)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::V8JS_GC_TYPE`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const V8JS_GC_DURATION: &str = "v8js.gc.duration";

/// ## Description
///
/// Heap space available size.
///
/// ## Notes
///
/// Value can be retrieved from value `space_available_size` of [`v8.getHeapSpaceStatistics()`](https://nodejs.org/api/v8.html#v8getheapspacestatistics)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::V8JS_HEAP_SPACE_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const V8JS_HEAP_SPACE_AVAILABLE_SIZE: &str = "v8js.heap.space.available_size";

/// ## Description
///
/// Committed size of a heap space.
///
/// ## Notes
///
/// Value can be retrieved from value `physical_space_size` of [`v8.getHeapSpaceStatistics()`](https://nodejs.org/api/v8.html#v8getheapspacestatistics)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::V8JS_HEAP_SPACE_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const V8JS_HEAP_SPACE_PHYSICAL_SIZE: &str = "v8js.heap.space.physical_size";

/// ## Description
///
/// Total heap memory size pre-allocated.
///
/// ## Notes
///
/// The value can be retrieved from value `space_size` of [`v8.getHeapSpaceStatistics()`](https://nodejs.org/api/v8.html#v8getheapspacestatistics)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::V8JS_HEAP_SPACE_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const V8JS_MEMORY_HEAP_LIMIT: &str = "v8js.memory.heap.limit";

/// ## Description
///
/// Heap Memory size allocated.
///
/// ## Notes
///
/// The value can be retrieved from value `space_used_size` of [`v8.getHeapSpaceStatistics()`](https://nodejs.org/api/v8.html#v8getheapspacestatistics)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::V8JS_HEAP_SPACE_NAME`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const V8JS_MEMORY_HEAP_USED: &str = "v8js.memory.heap.used";

/// ## Description
///
/// The number of changes (pull requests/merge requests/changelists) in a repository, categorized by their state (e.g. open or merged)
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{change}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::VCS_CHANGE_STATE`] | `Required`
/// | [`crate::attribute::VCS_REPOSITORY_URL_FULL`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const VCS_CHANGE_COUNT: &str = "vcs.change.count";

/// ## Description
///
/// The time duration a change (pull request/merge request/changelist) has been in a given state
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::VCS_CHANGE_STATE`] | `Required`
/// | [`crate::attribute::VCS_REF_HEAD_NAME`] | `Required`
/// | [`crate::attribute::VCS_REPOSITORY_URL_FULL`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const VCS_CHANGE_DURATION: &str = "vcs.change.duration";

/// ## Description
///
/// The amount of time since its creation it took a change (pull request/merge request/changelist) to get the first approval
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::VCS_REF_HEAD_NAME`] | `Required`
/// | [`crate::attribute::VCS_REPOSITORY_URL_FULL`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const VCS_CHANGE_TIME_TO_APPROVAL: &str = "vcs.change.time_to_approval";

/// ## Description
///
/// The number of unique contributors to a repository
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{contributor}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::VCS_REPOSITORY_URL_FULL`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const VCS_CONTRIBUTOR_COUNT: &str = "vcs.contributor.count";

/// ## Description
///
/// The number of refs of type branch or tag in a repository
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{ref}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::VCS_REF_TYPE`] | `Required`
/// | [`crate::attribute::VCS_REPOSITORY_URL_FULL`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const VCS_REF_COUNT: &str = "vcs.ref.count";

/// ## Description
///
/// The number of lines added/removed in a ref (branch) relative to the ref from the `vcs.ref.base.name` attribute
///
/// ## Notes
///
/// This metric should be reported for each `vcs.line_change.type` value. For example if a ref added 3 lines and removed 2 lines,
/// instrumentation SHOULD report two measurements: 3 and 2 (both positive numbers).
/// If number of lines added/removed should be calculated from the start of time, then `vcs.ref.base.name` SHOULD be set to an empty string
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{line}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::VCS_CHANGE_ID`] | `Conditionally_required`: if a change is associate with the ref.
/// | [`crate::attribute::VCS_LINE_CHANGE_TYPE`] | `Required`
/// | [`crate::attribute::VCS_REF_BASE_NAME`] | `Required`
/// | [`crate::attribute::VCS_REF_BASE_TYPE`] | `Required`
/// | [`crate::attribute::VCS_REF_HEAD_NAME`] | `Required`
/// | [`crate::attribute::VCS_REF_HEAD_TYPE`] | `Required`
/// | [`crate::attribute::VCS_REPOSITORY_URL_FULL`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const VCS_REF_LINES_DELTA: &str = "vcs.ref.lines_delta";

/// ## Description
///
/// The number of revisions (commits) a ref (branch) is ahead/behind the branch from the `vcs.ref.base.name` attribute
///
/// ## Notes
///
/// This metric should be reported for each `vcs.revision_delta.direction` value. For example if branch `a` is 3 commits behind and 2 commits ahead of `trunk`,
/// instrumentation SHOULD report two measurements: 3 and 2 (both positive numbers) and `vcs.ref.base.name` is set to `trunk`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{revision}` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::VCS_CHANGE_ID`] | `Conditionally_required`: if a change is associate with the ref.
/// | [`crate::attribute::VCS_REF_BASE_NAME`] | `Required`
/// | [`crate::attribute::VCS_REF_BASE_TYPE`] | `Required`
/// | [`crate::attribute::VCS_REF_HEAD_NAME`] | `Required`
/// | [`crate::attribute::VCS_REF_HEAD_TYPE`] | `Required`
/// | [`crate::attribute::VCS_REPOSITORY_URL_FULL`] | `Required`
/// | [`crate::attribute::VCS_REVISION_DELTA_DIRECTION`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const VCS_REF_REVISIONS_DELTA: &str = "vcs.ref.revisions_delta";

/// ## Description
///
/// Time a ref (branch) created from the default branch (trunk) has existed. The `ref.type` attribute will always be `branch`
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `s` |
/// | Status: | `Experimental`  |
///
/// ## Attributes
/// | Name | Requirement |
/// |:-|:- |
/// | [`crate::attribute::VCS_REF_HEAD_NAME`] | `Required`
/// | [`crate::attribute::VCS_REF_HEAD_TYPE`] | `Required`
/// | [`crate::attribute::VCS_REPOSITORY_URL_FULL`] | `Required`
#[cfg(feature = "semconv_experimental")]
pub const VCS_REF_TIME: &str = "vcs.ref.time";

/// ## Description
///
/// The number of repositories in an organization
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{repository}` |
/// | Status: | `Experimental`  |
#[cfg(feature = "semconv_experimental")]
pub const VCS_REPOSITORY_COUNT: &str = "vcs.repository.count";
