//! # Trace Semantic Conventions
//!
//! The [trace semantic conventions] define a set of standardized attributes to
//! be used in `Span`s.
//!
//! [trace semantic conventions]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/trace/semantic_conventions
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::trace::Tracer;
//! use opentelemetry::global;
//! use opentelemetry_semantic_conventions as semcov;
//!
//! let tracer = global::tracer("my-component");
//! let _span = tracer
//!     .span_builder("span-name")
//!     .with_attributes(vec![
//!         semcov::trace::NET_PEER_IP.string("10.0.0.1"),
//!         semcov::trace::NET_PEER_PORT.i64(80),
//!     ])
//!     .start(&tracer);
//! ```

use opentelemetry::Key;

/// An identifier for the database management system (DBMS) product being used.
pub const DB_SYSTEM: Key = Key::from_static_str("db.system");

/// The connection string used to connect to the database.
///
/// It is recommended to remove embedded credentials.
pub const DB_CONNECTION_STRING: Key = Key::from_static_str("db.connection_string");

/// Username for accessing the database.
pub const DB_USER: Key = Key::from_static_str("db.user");

/// Remote address of the peer (dotted decimal for IPv4 or [RFC5952] for IPv6)
///
/// [RFC5952]: https://tools.ietf.org/html/rfc5952
pub const NET_PEER_IP: Key = Key::from_static_str("net.peer.ip");

/// Remote hostname or similar.
pub const NET_PEER_NAME: Key = Key::from_static_str("net.peer.name");

/// Remote port number.
pub const NET_PEER_PORT: Key = Key::from_static_str("net.peer.port");

/// Transport protocol used.
pub const NET_TRANSPORT: Key = Key::from_static_str("net.transport");

/// The fully-qualified class name of the [Java Database Connectivity (JDBC)]
/// driver used to connect.
///
/// [Java Database Connectivity (JDBC)]: https://docs.oracle.com/javase/8/docs/technotes/guides/jdbc/
pub const DB_JDBC_DRIVER_CLASSNAME: Key = Key::from_static_str("db.jdbc.driver_classname");

/// The Microsoft SQL Server [instance name]) connecting to.
///
/// This name is used to determine the port of a named instance. If setting a
/// `db.mssql.instance_name`, `net.peer.port` is no longer required (but still
/// recommended if non-standard).
///
/// [instance name]: https://docs.microsoft.com/en-us/sql/connect/jdbc/building-the-connection-url?view=sql-server-ver15
pub const DB_MSSQL_INSTANCE_NAME: Key = Key::from_static_str("db.mssql.instance_name");

/// If no [tech-specific attribute] is defined, this attribute is used to report
/// the name of the database being accessed.
///
/// For commands that switch the database, this should be set to the target
/// database (even if the command fails).
///
/// In some SQL databases, the database name to be used is called "schema name".
///
/// [tech-specific attribute]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/trace/semantic_conventions/database.md#call-level-attributes-for-specific-technologies
pub const DB_NAME: Key = Key::from_static_str("db.name");

/// The database statement being executed.
///
/// The value may be sanitized to exclude sensitive information.
pub const DB_STATEMENT: Key = Key::from_static_str("db.statement");

/// The name of the operation being executed.
///
/// e.g. the [MongoDB command name] such as `findAndModify`.
///
/// While it would semantically make sense to set this, e.g., to a SQL keyword
/// like `SELECT` or `INSERT`, it is not recommended to attempt any client-side
/// parsing of `db.statement` just to get this property (the back end can do
/// that if required).
///
/// [MongoDB command name]: https://docs.mongodb.com/manual/reference/command/#database-operations
pub const DB_OPERATION: Key = Key::from_static_str("db.operation");

/// The name of the keyspace being accessed.
///
/// To be used instead of the generic `db.name` attribute.
pub const DB_CASSANDRA_KEYSPACE: Key = Key::from_static_str("db.cassandra.keyspace");

/// The [HBase namespace] being accessed.
///
/// To be used instead of the generic `db.name` attribute.
///
/// [HBase namespace]: https://hbase.apache.org/book.html#_namespace
pub const DB_HBASE_NAMESPACE: Key = Key::from_static_str("db.hbase.namespace");

/// The index of the database being accessed as used in the [`SELECT` command],
/// provided as an integer.
///
/// To be used instead of the generic `db.name` attribute.
///
/// [`SELECT` command]: https://redis.io/commands/select
pub const DB_REDIS_DATABASE_INDEX: Key = Key::from_static_str("db.redis.database_index");

/// The collection being accessed within the database stated in `db.name`.
pub const DB_MONGODB_COLLECTION: Key = Key::from_static_str("db.mongodb.collection");

/// The type of the exception (its fully-qualified class name, if applicable).
///
/// The dynamic type of the exception should be preferred over the static type
/// in languages that support it.
pub const EXCEPTION_TYPE: Key = Key::from_static_str("exception.type");

/// The exception message.
pub const EXCEPTION_MESSAGE: Key = Key::from_static_str("exception.message");

/// A stacktrace as a string in the natural representation for the language
/// runtime.
///
/// The representation is to be determined and documented by each language SIG.
pub const EXCEPTION_STACKTRACE: Key = Key::from_static_str("exception.stacktrace");

/// SHOULD be set to true if the exception event is recorded at a point where it
/// is known that the exception is escaping the scope of the span.
///
/// An exception is considered to have escaped (or left) the scope of a span, if
/// that span is ended while the exception is still logically "in flight". This
/// may be actually "in flight" in some languages (e.g. if the exception is
/// passed to a Context manager's `__exit__` method in Python) but will usually
/// be caught at the point of recording the exception in most languages.
///
/// It is usually not possible to determine at the point where an exception is
/// thrown whether it will escape the scope of a span. However, it is trivial to
/// know that an exception will escape, if one checks for an active exception
/// just before ending the span, as done in the [example
/// above](#exception-end-example).
///
/// It follows that an exception may still escape the scope of the span even if
/// the `exception.escaped` attribute was not set or set to false, since the
/// event might have been recorded at a time where it was not clear whether the
/// exception will escape.
pub const EXCEPTION_ESCAPED: Key = Key::from_static_str("exception.escaped");

/// Type of the trigger on which the function is executed.
///
/// It SHOULD be one of the following strings: "datasource", "http", "pubsub",
/// "timer", or "other".
pub const FAAS_TRIGGER: Key = Key::from_static_str("faas.trigger");

/// String containing the execution id of the function.
///
/// E.g. `af9d5aa4-a685-4c5f-a22b-444f80b3cc28`
pub const FAAS_EXECUTION: Key = Key::from_static_str("faas.execution");

/// Indicates that the serverless function is executed for the first time (aka
/// cold start).
pub const FAAS_COLDSTART: Key = Key::from_static_str("faas.coldstart");

/// The name of the invoked function.
pub const FAAS_INVOKED_NAME: Key = Key::from_static_str("faas.invoked_name");

/// The cloud provider of the invoked function.
pub const FAAS_INVOKED_PROVIDER: Key = Key::from_static_str("faas.invoked_provider");

/// The cloud region of the invoked function.
pub const FAAS_INVOKED_REGION: Key = Key::from_static_str("faas.invoked_region");

/// The name of the source on which the operation was performed.
///
/// For example, in Cloud Storage or S3 corresponds to the bucket name, and in
/// Cosmos DB to the database name.
pub const FAAS_DOCUMENT_COLLECTION: Key = Key::from_static_str("faas.document.collection");

/// Describes the type of the operation that was performed on the data.
///
/// It SHOULD be one of the following strings: "insert", "edit", "delete".
pub const FAAS_DOCUMENT_OPERATION: Key = Key::from_static_str("faas.document.operation");

/// A string containing the time when the data was accessed in the [ISO 8601]
/// format expressed in [UTC].
///
/// E.g. `"2020-01-23T13:47:06Z"`
///
/// [ISO 8601]: https://www.iso.org/iso-8601-date-and-time-format.html
/// [UTC]: https://www.w3.org/TR/NOTE-datetime
pub const FAAS_DOCUMENT_TIME: Key = Key::from_static_str("faas.document.time");

/// The document name/table subjected to the operation.
///
/// For example, in Cloud Storage or S3 is the name of the file, and in Cosmos
/// DB the table name.
pub const FAAS_DOCUMENT_NAME: Key = Key::from_static_str("faas.document.name");

/// A string containing the function invocation time in the [ISO 8601] format
/// expressed in [UTC].
///
/// E.g. `"2020-01-23T13:47:06Z"`
///
/// [ISO 8601]: https://www.iso.org/iso-8601-date-and-time-format.html
/// [UTC]: https://www.w3.org/TR/NOTE-datetime
pub const FAAS_TIME: Key = Key::from_static_str("faas.time");

/// A string containing the schedule period as [Cron Expression].
///
/// E.g. `"0/5 * * * ? *"`
///
/// [Cron Expression]: https://docs.oracle.com/cd/E12058_01/doc/doc.1014/e12030/cron_expressions.htm
pub const FAAS_CRON: Key = Key::from_static_str("faas.cron");

/// HTTP request method.
pub const HTTP_METHOD: Key = Key::from_static_str("http.method");

/// Full HTTP request URL in the form
/// `scheme://host[:port]/path?query[#fragment]`.
///
/// Usually the fragment is not transmitted over HTTP, but if it is known, it
/// should be included nevertheless.
pub const HTTP_URL: Key = Key::from_static_str("http.url");

/// The full request target as passed in a HTTP request line or equivalent.
pub const HTTP_TARGET: Key = Key::from_static_str("http.target");

/// The value of the [HTTP host header].
///
/// When the header is empty or not present, this attribute should be the same.
///
/// [HTTP host header]: https://tools.ietf.org/html/rfc7230#section-5.4
pub const HTTP_HOST: Key = Key::from_static_str("http.host");

/// The URI scheme identifying the used protocol.
pub const HTTP_SCHEME: Key = Key::from_static_str("http.scheme");

/// [HTTP response status code](https://tools.ietf.org/html/rfc7231#section-6).
pub const HTTP_STATUS_CODE: Key = Key::from_static_str("http.status_code");

/// Kind of HTTP protocol used.
///
/// If net.transport is not specified, it can be assumed to be IP.TCP except if
/// http.flavor is QUIC, in which case IP.UDP is assumed.
///
/// http.flavor` MUST be one of the following or, if none of the listed values
/// apply, a custom value:
///
/// | Value  | Description |
/// |---|---|
/// | `1.0` | HTTP 1.0 |
/// | `1.1` | HTTP 1.1 |
/// | `2.0` | HTTP 2 |
/// | `SPDY` | SPDY protocol. |
/// | `QUIC` | QUIC protocol. |
pub const HTTP_FLAVOR: Key = Key::from_static_str("http.flavor");

/// Value of the [HTTP User-Agent] header sent by the client.
///
/// [HTTP User-Agent]: https://tools.ietf.org/html/rfc7231#section-5.5.3
pub const HTTP_USER_AGENT: Key = Key::from_static_str("http.user_agent");

/// The size of the request payload body in bytes.
///
/// This is the number of bytes transferred excluding headers and is often, but
/// not always, present as the [Content-Length] header. For requests using
/// transport encoding, this should be the compressed size.
///
/// [Content-Length]: https://tools.ietf.org/html/rfc7230#section-3.3.2
pub const HTTP_REQUEST_CONTENT_LENGTH: Key = Key::from_static_str("http.request_content_length");

/// The size of the uncompressed request payload body after transport decoding.
///
/// Not set if transport encoding not used.
pub const HTTP_REQUEST_CONTENT_LENGTH_UNCOMPRESSED: Key =
    Key::from_static_str("http.request_content_length_uncompressed");

/// The size of the response payload body in bytes.
///
/// This is the number of bytes transferred excluding headers and is often, but
/// not always, present as the [Content-Length] header. For requests using
/// transport encoding, this should be the compressed size.
///
/// [Content-Length]: https://tools.ietf.org/html/rfc7230#section-3.3.2
pub const HTTP_RESPONSE_CONTENT_LENGTH: Key = Key::from_static_str("http.response_content_length");

/// The size of the uncompressed response payload body after transport decoding.
///
/// Not set if transport encoding not used.
pub const HTTP_RESPONSE_CONTENT_LENGTH_UNCOMPRESSED: Key =
    Key::from_static_str("http.response_content_length_uncompressed");

/// The primary server name of the matched virtual host.
///
/// This should be obtained via configuration. If no such configuration can be
/// obtained, this attribute MUST NOT be set ( `net.host.name` should be used
/// instead).
///
/// `http.url` is usually not readily available on the server side but would have
/// to be assembled in a cumbersome and sometimes lossy process from other
/// information (see e.g. open-telemetry/opentelemetry-python/pull/148). It is
/// thus preferred to supply the raw data that is available.
pub const HTTP_SERVER_NAME: Key = Key::from_static_str("http.server_name");

/// The matched route (path template).
pub const HTTP_ROUTE: Key = Key::from_static_str("http.route");

/// The IP address of the original client behind all proxies, if known (e.g.
/// from [X-Forwarded-For]).
///
/// This is not necessarily the same as `net.peer.ip`, which would identify the
/// network-level peer, which may be a proxy.
///
/// [X-Forwarded-For]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For
pub const HTTP_CLIENT_IP: Key = Key::from_static_str("http.client_ip");

/// A string identifying the messaging system.
pub const MESSAGING_SYSTEM: Key = Key::from_static_str("messaging.system");

/// The message destination name.
///
/// This might be equal to the span name but is required nevertheless.
pub const MESSAGING_DESTINATION: Key = Key::from_static_str("messaging.destination");

/// The kind of message destination
pub const MESSAGING_DESTINATION_KIND: Key = Key::from_static_str("messaging.destination_kind");

/// A boolean that is true if the message destination is temporary.
pub const MESSAGING_TEMP_DESTINATION: Key = Key::from_static_str("messaging.temp_destination");

/// The name of the transport protocol.
pub const MESSAGING_PROTOCOL: Key = Key::from_static_str("messaging.protocol");

/// The version of the transport protocol.
pub const MESSAGING_PROTOCOL_VERSION: Key = Key::from_static_str("messaging.protocol_version");

/// Connection string.
pub const MESSAGING_URL: Key = Key::from_static_str("messaging.url");

/// A value used by the messaging system as an identifier for the message,
/// represented as a string.
pub const MESSAGING_MESSAGE_ID: Key = Key::from_static_str("messaging.message_id");

/// The [conversation ID] identifying the conversation to which the message
/// belongs, represented as a string.
///
/// Sometimes called "Correlation ID".
///
/// [conversation ID]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/trace/semantic_conventions/messaging.md#conversations
pub const MESSAGING_CONVERSATION_ID: Key = Key::from_static_str("messaging.conversation_id");

/// The (uncompressed) size of the message payload in bytes.
///
/// Also use this attribute if it is unknown whether the compressed or
/// uncompressed payload size is reported.
pub const MESSAGING_MESSAGE_PAYLOAD_SIZE_BYTES: Key =
    Key::from_static_str("messaging.message_payload_size_bytes");

/// The compressed size of the message payload in bytes.
pub const MESSAGING_MESSAGE_PAYLOAD_COMPRESSED_SIZE_BYTES: Key =
    Key::from_static_str("messaging.message_payload_compressed_size_bytes");

/// A string identifying the kind of message consumption as defined in the
/// [Operation names] section above.
///
/// If the operation is "send", this attribute MUST NOT be set, since the
/// operation can be inferred from the span kind in that case.
///
/// [Operation names]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/trace/semantic_conventions/messaging.md#operation-names
pub const MESSAGING_OPERATION: Key = Key::from_static_str("messaging.operation");

/// A string identifying the remoting system.
pub const RPC_SYSTEM: Key = Key::from_static_str("rpc.system");

/// The full name of the service being called, including its package name, if
/// applicable.
pub const RPC_SERVICE: Key = Key::from_static_str("rpc.service");

/// The name of the method being called, must be equal to the $method part in
/// the span name.
pub const RPC_METHOD: Key = Key::from_static_str("rpc.method");

/// Like `net.peer.ip` but for the host IP.
///
/// Useful in case of a multi-IP host.
pub const NET_HOST_IP: Key = Key::from_static_str("net.host.ip");

/// Like `net.peer.port` but for the host port.
pub const NET_HOST_PORT: Key = Key::from_static_str("net.host.port");

/// Local hostname or similar
pub const NET_HOST_NAME: Key = Key::from_static_str("net.host.name");

/// The [`service.name`] of the remote service.
///
/// SHOULD be equal to the actual `service.name` resource attribute of the
/// remote service if any.
///
/// [`service.name`]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/resource/semantic_conventions/README.md#service
pub const PEER_SERVICE: Key = Key::from_static_str("peer.service");

/// Username or client_id extracted from the access token or [Authorization]
/// header in the inbound request from outside the system.
///
/// [Authorization]: https://tools.ietf.org/html/rfc7235#section-4.2
pub const ENDUSER_ID: Key = Key::from_static_str("enduser.id");

/// Actual/assumed role the client is making the request under extracted from
/// token or application security context.
pub const ENDUSER_ROLE: Key = Key::from_static_str("enduser.role");

/// Scopes or granted authorities the client currently possesses extracted from
/// token or application security context.
///
/// The value would come from the scope associated with an [OAuth 2.0 Access
/// Token] or an attribute value in a [SAML 2.0 Assertion].
///
/// [OAuth 2.0 Access Token]: https://tools.ietf.org/html/rfc6749#section-3.3
/// [SAML 2.0 Assertion]: http://docs.oasis-open.org/security/saml/Post2.0/sstc-saml-tech-overview-2.0.html
pub const ENDUSER_SCOPE: Key = Key::from_static_str("enduser.scope");

/// The method or function name, or equivalent (usually rightmost part of the
/// code unit's name).
pub const CODE_FUNCTION: Key = Key::from_static_str("code.function");

/// The "namespace" within which `code.function` is defined.
///
/// Usually the qualified class or module name, such that `code.namespace` +
/// some separator + `code.function` form a unique identifier for the code unit.
pub const CODE_NAMESPACE: Key = Key::from_static_str("code.namespace");

/// The source code file name that identifies the code unit as uniquely as
/// possible (preferably an absolute file path).
pub const CODE_FILEPATH: Key = Key::from_static_str("code.filepath");

/// The line number in `code.filepath` best representing the operation.
///
/// It SHOULD point within the code unit named in `code.function`.
pub const CODE_LINENO: Key = Key::from_static_str("code.lineno");
