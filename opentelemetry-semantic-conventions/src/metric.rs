// DO NOT EDIT, this is an auto-generated file
//
// If you want to update the file:
// - Edit the template at scripts/templates/semantic_metrics.rs.j2
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
//!     .init();
//! ```
/// ## Description
/// Number of exceptions caught by exception handling middleware.
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
/// | [`crate::attribute::ERROR_TYPE`] | `Required`
/// | [`crate::attribute::ASPNETCORE_DIAGNOSTICS_HANDLER_TYPE`] | `Conditionally required`: if and only if the exception was handled by this handler.
pub const ASPNETCORE_DIAGNOSTICS_EXCEPTIONS: &str = "aspnetcore.diagnostics.exceptions";
/// ## Description
/// Number of requests that are currently active on the server that hold a rate limiting lease.
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
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally required`: if the matched endpoint for the request had a rate-limiting policy.
pub const ASPNETCORE_RATE_LIMITING_ACTIVE_REQUEST_LEASES: &str =
    "aspnetcore.rate_limiting.active_request_leases";
/// ## Description
/// Number of requests that are currently queued, waiting to acquire a rate limiting lease.
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
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally required`: if the matched endpoint for the request had a rate-limiting policy.
pub const ASPNETCORE_RATE_LIMITING_QUEUED_REQUESTS: &str =
    "aspnetcore.rate_limiting.queued_requests";
/// ## Description
/// The time the request spent in a queue waiting to acquire a rate limiting lease.
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
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_RESULT`] | `Required`
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally required`: if the matched endpoint for the request had a rate-limiting policy.
pub const ASPNETCORE_RATE_LIMITING_REQUEST_TIME_IN_QUEUE: &str =
    "aspnetcore.rate_limiting.request.time_in_queue";
/// ## Description
/// The duration of rate limiting lease held by requests on the server.
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
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally required`: if the matched endpoint for the request had a rate-limiting policy.
pub const ASPNETCORE_RATE_LIMITING_REQUEST_LEASE_DURATION: &str =
    "aspnetcore.rate_limiting.request_lease.duration";
/// ## Description
/// Number of requests that tried to acquire a rate limiting lease.
///
/// Requests could be:
///
/// * Rejected by global or endpoint rate limiting policies
/// * Canceled while waiting for the lease.
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
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_RESULT`] | `Required`
/// | [`crate::attribute::ASPNETCORE_RATE_LIMITING_POLICY`] | `Conditionally required`: if the matched endpoint for the request had a rate-limiting policy.
pub const ASPNETCORE_RATE_LIMITING_REQUESTS: &str = "aspnetcore.rate_limiting.requests";
/// ## Description
/// Number of requests that were attempted to be matched to an endpoint.
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
/// | [`crate::attribute::ASPNETCORE_ROUTING_MATCH_STATUS`] | `Required`
/// | [`crate::attribute::ASPNETCORE_ROUTING_IS_FALLBACK`] | `Conditionally required`: if and only if a route was successfully matched.
/// | [`crate::attribute::HTTP_ROUTE`] | `Conditionally required`: if and only if a route was successfully matched.
pub const ASPNETCORE_ROUTING_MATCH_ATTEMPTS: &str = "aspnetcore.routing.match_attempts";
/// ## Description
/// Total CPU time consumed.
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
/// | [`crate::attribute::CONTAINER_CPU_STATE`] | `Opt in`
pub const CONTAINER_CPU_TIME: &str = "container.cpu.time";
/// ## Description
/// Disk bytes for the container.
///
/// The total number of bytes read/written successfully (aggregated from all disks).
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
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const CONTAINER_DISK_IO: &str = "container.disk.io";
/// ## Description
/// Memory usage of the container.
///
/// Memory usage of the container.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const CONTAINER_MEMORY_USAGE: &str = "container.memory.usage";
/// ## Description
/// Network bytes for the container.
///
/// The number of bytes sent/received on all network interfaces by the container.
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
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const CONTAINER_NETWORK_IO: &str = "container.network.io";
/// ## Description
/// The number of connections that are currently in state described by the `state` attribute.
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
pub const DB_CLIENT_CONNECTION_COUNT: &str = "db.client.connection.count";
/// ## Description
/// The time it took to create a new connection.
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
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
pub const DB_CLIENT_CONNECTION_CREATE_TIME: &str = "db.client.connection.create_time";
/// ## Description
/// The maximum number of idle open connections allowed.
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
pub const DB_CLIENT_CONNECTION_IDLE_MAX: &str = "db.client.connection.idle.max";
/// ## Description
/// The minimum number of idle open connections allowed.
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
pub const DB_CLIENT_CONNECTION_IDLE_MIN: &str = "db.client.connection.idle.min";
/// ## Description
/// The maximum number of open connections allowed.
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
pub const DB_CLIENT_CONNECTION_MAX: &str = "db.client.connection.max";
/// ## Description
/// The number of pending requests for an open connection, cumulative for the entire pool.
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
pub const DB_CLIENT_CONNECTION_PENDING_REQUESTS: &str = "db.client.connection.pending_requests";
/// ## Description
/// The number of connection timeouts that have occurred trying to obtain a connection from the pool.
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
pub const DB_CLIENT_CONNECTION_TIMEOUTS: &str = "db.client.connection.timeouts";
/// ## Description
/// The time between borrowing a connection and returning it to the pool.
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
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
pub const DB_CLIENT_CONNECTION_USE_TIME: &str = "db.client.connection.use_time";
/// ## Description
/// The time it took to obtain an open connection from the pool.
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
/// | [`crate::attribute::DB_CLIENT_CONNECTIONS_POOL_NAME`] | `Required`
pub const DB_CLIENT_CONNECTION_WAIT_TIME: &str = "db.client.connection.wait_time";
/// ## Description
/// Deprecated, use `db.client.connection.create_time` instead. Note: the unit also changed from `ms` to `s`.
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
pub const DB_CLIENT_CONNECTIONS_CREATE_TIME: &str = "db.client.connections.create_time";
/// ## Description
/// Deprecated, use `db.client.connection.idle.max` instead.
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
pub const DB_CLIENT_CONNECTIONS_IDLE_MAX: &str = "db.client.connections.idle.max";
/// ## Description
/// Deprecated, use `db.client.connection.idle.min` instead.
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
pub const DB_CLIENT_CONNECTIONS_IDLE_MIN: &str = "db.client.connections.idle.min";
/// ## Description
/// Deprecated, use `db.client.connection.max` instead.
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
pub const DB_CLIENT_CONNECTIONS_MAX: &str = "db.client.connections.max";
/// ## Description
/// Deprecated, use `db.client.connection.pending_requests` instead.
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
pub const DB_CLIENT_CONNECTIONS_PENDING_REQUESTS: &str = "db.client.connections.pending_requests";
/// ## Description
/// Deprecated, use `db.client.connection.timeouts` instead.
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
pub const DB_CLIENT_CONNECTIONS_TIMEOUTS: &str = "db.client.connections.timeouts";
/// ## Description
/// Deprecated, use `db.client.connection.count` instead.
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
pub const DB_CLIENT_CONNECTIONS_USAGE: &str = "db.client.connections.usage";
/// ## Description
/// Deprecated, use `db.client.connection.use_time` instead. Note: the unit also changed from `ms` to `s`.
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
pub const DB_CLIENT_CONNECTIONS_USE_TIME: &str = "db.client.connections.use_time";
/// ## Description
/// Deprecated, use `db.client.connection.wait_time` instead. Note: the unit also changed from `ms` to `s`.
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
pub const DB_CLIENT_CONNECTIONS_WAIT_TIME: &str = "db.client.connections.wait_time";
/// ## Description
/// Duration of database client operations.
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
/// | [`crate::attribute::DB_SYSTEM`] | `Required`
/// | [`crate::attribute::DB_COLLECTION_NAME`] | `Conditionally required`: If readily available. Otherwise, if the instrumentation library parses `db.query.text` to capture `db.collection.name`, then it SHOULD be the first collection name found in the query.
/// | [`crate::attribute::DB_NAMESPACE`] | `Conditionally required`: If available.
/// | [`crate::attribute::DB_OPERATION_NAME`] | `Conditionally required`: If readily available. Otherwise, if the instrumentation library parses `db.query.text` to capture `db.operation.name`, then it SHOULD be the first operation name found in the query.
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If and only if the operation failed.
/// | [`crate::attribute::SERVER_PORT`] | `Conditionally required`: If using a port other than the default port for this DBMS and if `server.address` is set.
/// | [`crate::attribute::NETWORK_PEER_ADDRESS`] | `Recommended`: If applicable for this database system.
/// | [`crate::attribute::NETWORK_PEER_PORT`] | `Recommended`: If and only if `network.peer.address` is set.
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
pub const DB_CLIENT_OPERATION_DURATION: &str = "db.client.operation.duration";
/// ## Description
/// Measures the time taken to perform a DNS lookup.
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
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: if and only if an error has occurred.
pub const DNS_LOOKUP_DURATION: &str = "dns.lookup.duration";
/// ## Description
/// Number of invocation cold starts.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_COLDSTARTS: &str = "faas.coldstarts";
/// ## Description
/// Distribution of CPU usage per invocation.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_CPU_USAGE: &str = "faas.cpu_usage";
/// ## Description
/// Number of invocation errors.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_ERRORS: &str = "faas.errors";
/// ## Description
/// Measures the duration of the function&#39;s initialization, such as a cold start.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_INIT_DURATION: &str = "faas.init_duration";
/// ## Description
/// Number of successful invocations.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_INVOCATIONS: &str = "faas.invocations";
/// ## Description
/// Measures the duration of the function&#39;s logic execution.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_INVOKE_DURATION: &str = "faas.invoke_duration";
/// ## Description
/// Distribution of max memory usage per invocation.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_MEM_USAGE: &str = "faas.mem_usage";
/// ## Description
/// Distribution of net I/O usage per invocation.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_NET_IO: &str = "faas.net_io";
/// ## Description
/// Number of invocation timeouts.
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
/// | [`crate::attribute::FAAS_TRIGGER`] | `Unspecified`
pub const FAAS_TIMEOUTS: &str = "faas.timeouts";
/// ## Description
/// Number of active HTTP requests.
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
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::URL_TEMPLATE`] | `Conditionally required`: If available.
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Recommended`
/// | [`crate::attribute::URL_SCHEME`] | `Opt in`
pub const HTTP_CLIENT_ACTIVE_REQUESTS: &str = "http.client.active_requests";
/// ## Description
/// The duration of the successfully established outbound HTTP connections.
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
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::NETWORK_PEER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::URL_SCHEME`] | `Opt in`
pub const HTTP_CLIENT_CONNECTION_DURATION: &str = "http.client.connection.duration";
/// ## Description
/// Number of outbound HTTP connections that are currently active or idle on the client.
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
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::NETWORK_PEER_ADDRESS`] | `Recommended`
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Recommended`
/// | [`crate::attribute::URL_SCHEME`] | `Opt in`
pub const HTTP_CLIENT_OPEN_CONNECTIONS: &str = "http.client.open_connections";
/// ## Description
/// Size of HTTP client request bodies.
///
/// The size of the request payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
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
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally required`: If and only if one was received/sent.
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::URL_TEMPLATE`] | `Conditionally required`: If available.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Unspecified`
/// | [`crate::attribute::URL_SCHEME`] | `Opt in`
pub const HTTP_CLIENT_REQUEST_BODY_SIZE: &str = "http.client.request.body.size";
/// ## Description
/// Duration of HTTP client requests.
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
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally required`: If and only if one was received/sent.
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Unspecified`
/// | [`crate::attribute::URL_SCHEME`] | `Opt in`
pub const HTTP_CLIENT_REQUEST_DURATION: &str = "http.client.request.duration";
/// ## Description
/// Size of HTTP client response bodies.
///
/// The size of the response payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
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
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Required`
/// | [`crate::attribute::SERVER_PORT`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally required`: If and only if one was received/sent.
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::URL_TEMPLATE`] | `Conditionally required`: If available.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Unspecified`
/// | [`crate::attribute::URL_SCHEME`] | `Opt in`
pub const HTTP_CLIENT_RESPONSE_BODY_SIZE: &str = "http.client.response.body.size";
/// ## Description
/// Number of active HTTP server requests.
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
/// | [`crate::attribute::URL_SCHEME`] | `Required`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Opt in`
/// | [`crate::attribute::SERVER_PORT`] | `Opt in`
pub const HTTP_SERVER_ACTIVE_REQUESTS: &str = "http.server.active_requests";
/// ## Description
/// Size of HTTP server request bodies.
///
/// The size of the request payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
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
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally required`: If and only if one was received/sent.
/// | [`crate::attribute::HTTP_ROUTE`] | `Conditionally required`: If and only if it's available
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Unspecified`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Opt in`
/// | [`crate::attribute::SERVER_PORT`] | `Opt in`
pub const HTTP_SERVER_REQUEST_BODY_SIZE: &str = "http.server.request.body.size";
/// ## Description
/// Duration of HTTP server requests.
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
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally required`: If and only if one was received/sent.
/// | [`crate::attribute::HTTP_ROUTE`] | `Conditionally required`: If and only if it's available
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Unspecified`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Opt in`
/// | [`crate::attribute::SERVER_PORT`] | `Opt in`
pub const HTTP_SERVER_REQUEST_DURATION: &str = "http.server.request.duration";
/// ## Description
/// Size of HTTP server response bodies.
///
/// The size of the response payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
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
/// | [`crate::attribute::HTTP_REQUEST_METHOD`] | `Required`
/// | [`crate::attribute::URL_SCHEME`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If request has ended with an error.
/// | [`crate::attribute::HTTP_RESPONSE_STATUS_CODE`] | `Conditionally required`: If and only if one was received/sent.
/// | [`crate::attribute::HTTP_ROUTE`] | `Conditionally required`: If and only if it's available
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Conditionally required`: If not `http` and `network.protocol.version` is set.
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Unspecified`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Opt in`
/// | [`crate::attribute::SERVER_PORT`] | `Opt in`
pub const HTTP_SERVER_RESPONSE_BODY_SIZE: &str = "http.server.response.body.size";
/// ## Description
/// Number of buffers in the pool.
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
pub const JVM_BUFFER_COUNT: &str = "jvm.buffer.count";
/// ## Description
/// Measure of total memory capacity of buffers.
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
pub const JVM_BUFFER_MEMORY_LIMIT: &str = "jvm.buffer.memory.limit";
/// ## Description
/// Measure of memory used by buffers.
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
pub const JVM_BUFFER_MEMORY_USAGE: &str = "jvm.buffer.memory.usage";
/// ## Description
/// Number of classes currently loaded.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{class}` |
/// | Status: | `Stable`  |
pub const JVM_CLASS_COUNT: &str = "jvm.class.count";
/// ## Description
/// Number of classes loaded since JVM start.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{class}` |
/// | Status: | `Stable`  |
pub const JVM_CLASS_LOADED: &str = "jvm.class.loaded";
/// ## Description
/// Number of classes unloaded since JVM start.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{class}` |
/// | Status: | `Stable`  |
pub const JVM_CLASS_UNLOADED: &str = "jvm.class.unloaded";
/// ## Description
/// Number of processors available to the Java virtual machine.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{cpu}` |
/// | Status: | `Stable`  |
pub const JVM_CPU_COUNT: &str = "jvm.cpu.count";
/// ## Description
/// Recent CPU utilization for the process as reported by the JVM.
///
/// The value range is \[0.0,1.0\]. This utilization is not defined as being for the specific interval since last measurement (unlike `system.cpu.utilization`). [Reference](https://docs.oracle.com/en/java/javase/17/docs/api/jdk.management/com/sun/management/OperatingSystemMXBean.html#getProcessCpuLoad()).
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Stable`  |
pub const JVM_CPU_RECENT_UTILIZATION: &str = "jvm.cpu.recent_utilization";
/// ## Description
/// CPU time used by the process as reported by the JVM.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `s` |
/// | Status: | `Stable`  |
pub const JVM_CPU_TIME: &str = "jvm.cpu.time";
/// ## Description
/// Duration of JVM garbage collection actions.
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
/// Measure of memory committed.
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
/// Measure of initial memory requested.
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
pub const JVM_MEMORY_INIT: &str = "jvm.memory.init";
/// ## Description
/// Measure of max obtainable memory.
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
/// Measure of memory used.
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
/// Measure of memory used, as measured after the most recent garbage collection event on this pool.
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
/// Average CPU load of the whole system for the last minute as reported by the JVM.
///
/// The value range is \[0,n\], where n is the number of CPU cores - or a negative number if the value is not available. This utilization is not defined as being for the specific interval since last measurement (unlike `system.cpu.utilization`). [Reference](https://docs.oracle.com/en/java/javase/17/docs/api/java.management/java/lang/management/OperatingSystemMXBean.html#getSystemLoadAverage()).
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `{run_queue_item}` |
/// | Status: | `Experimental`  |
pub const JVM_SYSTEM_CPU_LOAD_1M: &str = "jvm.system.cpu.load_1m";
/// ## Description
/// Recent CPU utilization for the whole system as reported by the JVM.
///
/// The value range is \[0.0,1.0\]. This utilization is not defined as being for the specific interval since last measurement (unlike `system.cpu.utilization`). [Reference](https://docs.oracle.com/en/java/javase/17/docs/api/jdk.management/com/sun/management/OperatingSystemMXBean.html#getCpuLoad()).
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `gauge` |
/// | Unit: | `1` |
/// | Status: | `Experimental`  |
pub const JVM_SYSTEM_CPU_UTILIZATION: &str = "jvm.system.cpu.utilization";
/// ## Description
/// Number of executing platform threads.
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
/// Number of connections that are currently active on the server.
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
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TYPE`] | `Recommended`: if the transport is `tcp` or `udp`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const KESTREL_ACTIVE_CONNECTIONS: &str = "kestrel.active_connections";
/// ## Description
/// Number of TLS handshakes that are currently in progress on the server.
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
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TYPE`] | `Recommended`: if the transport is `tcp` or `udp`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const KESTREL_ACTIVE_TLS_HANDSHAKES: &str = "kestrel.active_tls_handshakes";
/// ## Description
/// The duration of connections on the server.
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
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: if and only if an error has occurred.
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Unspecified`
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TYPE`] | `Recommended`: if the transport is `tcp` or `udp`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
/// | [`crate::attribute::TLS_PROTOCOL_VERSION`] | `Unspecified`
pub const KESTREL_CONNECTION_DURATION: &str = "kestrel.connection.duration";
/// ## Description
/// Number of connections that are currently queued and are waiting to start.
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
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TYPE`] | `Recommended`: if the transport is `tcp` or `udp`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const KESTREL_QUEUED_CONNECTIONS: &str = "kestrel.queued_connections";
/// ## Description
/// Number of HTTP requests on multiplexed connections (HTTP/2 and HTTP/3) that are currently queued and are waiting to start.
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
/// | [`crate::attribute::NETWORK_PROTOCOL_NAME`] | `Unspecified`
/// | [`crate::attribute::NETWORK_PROTOCOL_VERSION`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TYPE`] | `Recommended`: if the transport is `tcp` or `udp`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const KESTREL_QUEUED_REQUESTS: &str = "kestrel.queued_requests";
/// ## Description
/// Number of connections rejected by the server.
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
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TYPE`] | `Recommended`: if the transport is `tcp` or `udp`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const KESTREL_REJECTED_CONNECTIONS: &str = "kestrel.rejected_connections";
/// ## Description
/// The duration of TLS handshakes on the server.
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
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: if and only if an error has occurred.
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TYPE`] | `Recommended`: if the transport is `tcp` or `udp`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
/// | [`crate::attribute::TLS_PROTOCOL_VERSION`] | `Unspecified`
pub const KESTREL_TLS_HANDSHAKE_DURATION: &str = "kestrel.tls_handshake.duration";
/// ## Description
/// Number of connections that are currently upgraded (WebSockets). .
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
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::NETWORK_TYPE`] | `Recommended`: if the transport is `tcp` or `udp`
/// | [`crate::attribute::SERVER_ADDRESS`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const KESTREL_UPGRADED_CONNECTIONS: &str = "kestrel.upgraded_connections";
/// ## Description
/// Measures the duration of process operation.
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
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally required`: if available.
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally required`: If available.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const MESSAGING_PROCESS_DURATION: &str = "messaging.process.duration";
/// ## Description
/// Measures the number of processed messages.
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
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally required`: if available.
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally required`: If available.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const MESSAGING_PROCESS_MESSAGES: &str = "messaging.process.messages";
/// ## Description
/// Measures the duration of publish operation.
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
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally required`: if available.
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally required`: If available.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const MESSAGING_PUBLISH_DURATION: &str = "messaging.publish.duration";
/// ## Description
/// Measures the number of published messages.
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
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally required`: if available.
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally required`: If available.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const MESSAGING_PUBLISH_MESSAGES: &str = "messaging.publish.messages";
/// ## Description
/// Measures the duration of receive operation.
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
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally required`: if available.
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally required`: If available.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const MESSAGING_RECEIVE_DURATION: &str = "messaging.receive.duration";
/// ## Description
/// Measures the number of received messages.
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
/// | [`crate::attribute::MESSAGING_SYSTEM`] | `Required`
/// | [`crate::attribute::ERROR_TYPE`] | `Conditionally required`: If and only if the messaging operation has failed.
/// | [`crate::attribute::MESSAGING_DESTINATION_NAME`] | `Conditionally required`: if and only if `messaging.destination.name` is known to have low cardinality. Otherwise, `messaging.destination.template` MAY be populated.
/// | [`crate::attribute::MESSAGING_DESTINATION_TEMPLATE`] | `Conditionally required`: if available.
/// | [`crate::attribute::SERVER_ADDRESS`] | `Conditionally required`: If available.
/// | [`crate::attribute::MESSAGING_DESTINATION_PARTITION_ID`] | `Unspecified`
/// | [`crate::attribute::SERVER_PORT`] | `Unspecified`
pub const MESSAGING_RECEIVE_MESSAGES: &str = "messaging.receive.messages";
/// ## Description
/// Number of times the process has been context switched.
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
/// | [`crate::attribute::PROCESS_CONTEXT_SWITCH_TYPE`] | `Unspecified`
pub const PROCESS_CONTEXT_SWITCHES: &str = "process.context_switches";
/// ## Description
/// Total CPU seconds broken down by different states.
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
/// | [`crate::attribute::PROCESS_CPU_STATE`] | `Unspecified`
pub const PROCESS_CPU_TIME: &str = "process.cpu.time";
/// ## Description
/// Difference in process.cpu.time since the last measurement, divided by the elapsed time and number of CPUs available to the process.
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
/// | [`crate::attribute::PROCESS_CPU_STATE`] | `Unspecified`
pub const PROCESS_CPU_UTILIZATION: &str = "process.cpu.utilization";
/// ## Description
/// Disk bytes transferred.
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
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Unspecified`
pub const PROCESS_DISK_IO: &str = "process.disk.io";
/// ## Description
/// The amount of physical memory in use.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const PROCESS_MEMORY_USAGE: &str = "process.memory.usage";
/// ## Description
/// The amount of committed virtual memory.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const PROCESS_MEMORY_VIRTUAL: &str = "process.memory.virtual";
/// ## Description
/// Network bytes transferred.
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
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Unspecified`
pub const PROCESS_NETWORK_IO: &str = "process.network.io";
/// ## Description
/// Number of file descriptors in use by the process.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{count}` |
/// | Status: | `Experimental`  |
pub const PROCESS_OPEN_FILE_DESCRIPTOR_COUNT: &str = "process.open_file_descriptor.count";
/// ## Description
/// Number of page faults the process has made.
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
/// | [`crate::attribute::PROCESS_PAGING_FAULT_TYPE`] | `Unspecified`
pub const PROCESS_PAGING_FAULTS: &str = "process.paging.faults";
/// ## Description
/// Process threads count.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{thread}` |
/// | Status: | `Experimental`  |
pub const PROCESS_THREAD_COUNT: &str = "process.thread.count";
/// ## Description
/// Measures the duration of outbound RPC.
///
/// While streaming RPCs may record this metric as start-of-batch
/// to end-of-batch, it&#39;s hard to interpret in practice.
///
/// **Streaming**: N/A.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `ms` |
/// | Status: | `Experimental`  |
pub const RPC_CLIENT_DURATION: &str = "rpc.client.duration";
/// ## Description
/// Measures the size of RPC request messages (uncompressed).
///
/// **Streaming**: Recorded per message in a streaming batch
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const RPC_CLIENT_REQUEST_SIZE: &str = "rpc.client.request.size";
/// ## Description
/// Measures the number of messages received per RPC.
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
pub const RPC_CLIENT_REQUESTS_PER_RPC: &str = "rpc.client.requests_per_rpc";
/// ## Description
/// Measures the size of RPC response messages (uncompressed).
///
/// **Streaming**: Recorded per response in a streaming batch
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const RPC_CLIENT_RESPONSE_SIZE: &str = "rpc.client.response.size";
/// ## Description
/// Measures the number of messages sent per RPC.
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
pub const RPC_CLIENT_RESPONSES_PER_RPC: &str = "rpc.client.responses_per_rpc";
/// ## Description
/// Measures the duration of inbound RPC.
///
/// While streaming RPCs may record this metric as start-of-batch
/// to end-of-batch, it&#39;s hard to interpret in practice.
///
/// **Streaming**: N/A.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `ms` |
/// | Status: | `Experimental`  |
pub const RPC_SERVER_DURATION: &str = "rpc.server.duration";
/// ## Description
/// Measures the size of RPC request messages (uncompressed).
///
/// **Streaming**: Recorded per message in a streaming batch
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const RPC_SERVER_REQUEST_SIZE: &str = "rpc.server.request.size";
/// ## Description
/// Measures the number of messages received per RPC.
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
pub const RPC_SERVER_REQUESTS_PER_RPC: &str = "rpc.server.requests_per_rpc";
/// ## Description
/// Measures the size of RPC response messages (uncompressed).
///
/// **Streaming**: Recorded per response in a streaming batch
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `histogram` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const RPC_SERVER_RESPONSE_SIZE: &str = "rpc.server.response.size";
/// ## Description
/// Measures the number of messages sent per RPC.
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
pub const RPC_SERVER_RESPONSES_PER_RPC: &str = "rpc.server.responses_per_rpc";
/// ## Description
/// Number of connections that are currently active on the server.
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
/// | [`crate::attribute::SIGNALR_CONNECTION_STATUS`] | `Unspecified`
/// | [`crate::attribute::SIGNALR_TRANSPORT`] | `Unspecified`
pub const SIGNALR_SERVER_ACTIVE_CONNECTIONS: &str = "signalr.server.active_connections";
/// ## Description
/// The duration of connections on the server.
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
/// | [`crate::attribute::SIGNALR_CONNECTION_STATUS`] | `Unspecified`
/// | [`crate::attribute::SIGNALR_TRANSPORT`] | `Unspecified`
pub const SIGNALR_SERVER_CONNECTION_DURATION: &str = "signalr.server.connection.duration";
/// ## Description
/// Reports the current frequency of the CPU in Hz.
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
/// | [`crate::attribute::SYSTEM_CPU_LOGICAL_NUMBER`] | `Unspecified`
pub const SYSTEM_CPU_FREQUENCY: &str = "system.cpu.frequency";
/// ## Description
/// Reports the number of logical (virtual) processor cores created by the operating system to manage multitasking.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{cpu}` |
/// | Status: | `Experimental`  |
pub const SYSTEM_CPU_LOGICAL_COUNT: &str = "system.cpu.logical.count";
/// ## Description
/// Reports the number of actual physical processor cores on the hardware.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `{cpu}` |
/// | Status: | `Experimental`  |
pub const SYSTEM_CPU_PHYSICAL_COUNT: &str = "system.cpu.physical.count";
/// ## Description
/// Seconds each logical CPU spent on each mode.
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
/// | [`crate::attribute::SYSTEM_CPU_LOGICAL_NUMBER`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_CPU_STATE`] | `Unspecified`
pub const SYSTEM_CPU_TIME: &str = "system.cpu.time";
/// ## Description
/// Difference in system.cpu.time since the last measurement, divided by the elapsed time and number of logical CPUs.
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
/// | [`crate::attribute::SYSTEM_CPU_LOGICAL_NUMBER`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_CPU_STATE`] | `Unspecified`
pub const SYSTEM_CPU_UTILIZATION: &str = "system.cpu.utilization";
/// ## Description
/// .
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
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_DISK_IO: &str = "system.disk.io";
/// ## Description
/// Time disk spent activated.
///
/// The real elapsed time (&#34;wall clock&#34;) used in the I/O path (time from operations running in parallel are not counted). Measured as:
///
/// - Linux: Field 13 from [procfs-diskstats](https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats)
/// - Windows: The complement of
///   [&#34;Disk\% Idle Time&#34;](https://learn.microsoft.com/archive/blogs/askcore/windows-performance-monitor-disk-counters-explained#windows-performance-monitor-disk-counters-explained)
///   performance counter: `uptime * (100 - &#34;Disk\% Idle Time&#34;) / 100`
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
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_DISK_IO_TIME: &str = "system.disk.io_time";
/// ## Description
/// .
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
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_DISK_MERGED: &str = "system.disk.merged";
/// ## Description
/// Sum of the time each operation took to complete.
///
/// Because it is the sum of time each request took, parallel-issued requests each contribute to make the count grow. Measured as:
///
/// - Linux: Fields 7 &amp; 11 from [procfs-diskstats](https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats)
/// - Windows: &#34;Avg. Disk sec/Read&#34; perf counter multiplied by &#34;Disk Reads/sec&#34; perf counter (similar for Writes)
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
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_DISK_OPERATION_TIME: &str = "system.disk.operation_time";
/// ## Description
/// .
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
/// | [`crate::attribute::DISK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_DISK_OPERATIONS: &str = "system.disk.operations";
/// ## Description
/// .
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
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MODE`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MOUNTPOINT`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_STATE`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_TYPE`] | `Unspecified`
pub const SYSTEM_FILESYSTEM_USAGE: &str = "system.filesystem.usage";
/// ## Description
/// .
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
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MODE`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_MOUNTPOINT`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_STATE`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_FILESYSTEM_TYPE`] | `Unspecified`
pub const SYSTEM_FILESYSTEM_UTILIZATION: &str = "system.filesystem.utilization";
/// ## Description
/// An estimate of how much memory is available for starting new applications, without causing swapping.
///
/// This is an alternative to `system.memory.usage` metric with `state=free`.
/// Linux starting from 3.14 exports &#34;available&#34; memory. It takes &#34;free&#34; memory as a baseline, and then factors in kernel-specific values.
/// This is supposed to be more accurate than just &#34;free&#34; memory.
/// For reference, see the calculations [here](https://superuser.com/a/980821).
/// See also `MemAvailable` in [/proc/meminfo](https://man7.org/linux/man-pages/man5/proc.5.html).
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const SYSTEM_LINUX_MEMORY_AVAILABLE: &str = "system.linux.memory.available";
/// ## Description
/// Total memory available in the system.
///
/// Its value SHOULD equal the sum of `system.memory.state` over all states.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const SYSTEM_MEMORY_LIMIT: &str = "system.memory.limit";
/// ## Description
/// Shared memory used (mostly by tmpfs).
///
/// Equivalent of `shared` from [`free` command](https://man7.org/linux/man-pages/man1/free.1.html) or
/// `Shmem` from [`/proc/meminfo`](https://man7.org/linux/man-pages/man5/proc.5.html)&#34;
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `updowncounter` |
/// | Unit: | `By` |
/// | Status: | `Experimental`  |
pub const SYSTEM_MEMORY_SHARED: &str = "system.memory.shared";
/// ## Description
/// Reports memory in use by state.
///
/// The sum over all `system.memory.state` values SHOULD equal the total memory
/// available on the system, that is `system.memory.limit`.
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
/// | [`crate::attribute::SYSTEM_MEMORY_STATE`] | `Unspecified`
pub const SYSTEM_MEMORY_USAGE: &str = "system.memory.usage";
/// ## Description
/// .
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
/// | [`crate::attribute::SYSTEM_MEMORY_STATE`] | `Unspecified`
pub const SYSTEM_MEMORY_UTILIZATION: &str = "system.memory.utilization";
/// ## Description
/// .
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
/// | [`crate::attribute::NETWORK_TRANSPORT`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_NETWORK_STATE`] | `Unspecified`
pub const SYSTEM_NETWORK_CONNECTIONS: &str = "system.network.connections";
/// ## Description
/// Count of packets that are dropped or discarded even though there was no error.
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
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_NETWORK_DROPPED: &str = "system.network.dropped";
/// ## Description
/// Count of network errors detected.
///
/// Measured as:
///
/// - Linux: the `errs` column in `/proc/dev/net` ([source](https://web.archive.org/web/20180321091318/http://www.onlamp.com/pub/a/linux/2000/11/16/LinuxAdmin.html)).
/// - Windows: [`InErrors`/`OutErrors`](https://docs.microsoft.com/windows/win32/api/netioapi/ns-netioapi-mib_if_row2)
///   from [`GetIfEntry2`](https://docs.microsoft.com/windows/win32/api/netioapi/nf-netioapi-getifentry2).
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
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_NETWORK_ERRORS: &str = "system.network.errors";
/// ## Description
/// .
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
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_NETWORK_IO: &str = "system.network.io";
/// ## Description
/// .
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
/// | [`crate::attribute::NETWORK_IO_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_DEVICE`] | `Unspecified`
pub const SYSTEM_NETWORK_PACKETS: &str = "system.network.packets";
/// ## Description
/// .
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
/// | [`crate::attribute::SYSTEM_PAGING_TYPE`] | `Unspecified`
pub const SYSTEM_PAGING_FAULTS: &str = "system.paging.faults";
/// ## Description
/// .
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
/// | [`crate::attribute::SYSTEM_PAGING_DIRECTION`] | `Unspecified`
/// | [`crate::attribute::SYSTEM_PAGING_TYPE`] | `Unspecified`
pub const SYSTEM_PAGING_OPERATIONS: &str = "system.paging.operations";
/// ## Description
/// Unix swap or windows pagefile usage.
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
/// | [`crate::attribute::SYSTEM_PAGING_STATE`] | `Unspecified`
pub const SYSTEM_PAGING_USAGE: &str = "system.paging.usage";
/// ## Description
/// .
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
/// | [`crate::attribute::SYSTEM_PAGING_STATE`] | `Unspecified`
pub const SYSTEM_PAGING_UTILIZATION: &str = "system.paging.utilization";
/// ## Description
/// Total number of processes in each state.
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
/// | [`crate::attribute::SYSTEM_PROCESS_STATUS`] | `Unspecified`
pub const SYSTEM_PROCESS_COUNT: &str = "system.process.count";
/// ## Description
/// Total number of processes created over uptime of the host.
/// ## Metadata
/// | | |
/// |:-|:-
/// | Instrument: | `counter` |
/// | Unit: | `{process}` |
/// | Status: | `Experimental`  |
pub const SYSTEM_PROCESS_CREATED: &str = "system.process.created";
