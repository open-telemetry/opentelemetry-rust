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
//! use opentelemetry::api::Tracer;
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

use opentelemetry::api::Key;

/// Transport protocol used.
pub const NET_TRANSPORT: Key = Key::from_static_str("net.transport");

/// Remote address of the peer (dotted decimal for IPv4 or [RFC5952] for IPv6)
///
/// [RFC5952]: https://tools.ietf.org/html/rfc5952
pub const NET_PEER_IP: Key = Key::from_static_str("net.peer.ip");

/// Remote port number as an integer. E.g., `80`.
pub const NET_PEER_PORT: Key = Key::from_static_str("net.peer.port");

/// Remote hostname or similar.                          
pub const NET_PEER_NAME: Key = Key::from_static_str("net.peer.name");

/// Like `net.peer.ip` but for the host IP. Useful in case of a multi-IP host.        
pub const NET_HOST_IP: Key = Key::from_static_str("net.host.ip");

/// Like `net.peer.port` but for the host port.                                       
pub const NET_HOST_PORT: Key = Key::from_static_str("net.host.port");

/// Local hostname or similar.
pub const NET_HOST_NAME: Key = Key::from_static_str("net.host.name");

/// The `service.name` of the remote service. SHOULD be equal to the actual
/// `service.name` resource attribute of the remote service if any.
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
/// token or application security context. The value would come from the scope
/// associated with an [OAuth 2.0 Access Token] or an attribute value in a [SAML
/// 2.0 Assertion].
///
/// [OAuth 2.0 Access Token]: https://tools.ietf.org/html/rfc6749#section-3.3
/// [SAML 2.0 Assertion]: http://docs.oasis-open.org/security/saml/Post2.0/sstc-saml-tech-overview-2.0.html
pub const ENDUSER_SCOPE: Key = Key::from_static_str("enduser.scope");

/// Current "managed" thread ID (as opposed to OS thread ID). E.g. `42`
pub const THREAD_ID: Key = Key::from_static_str("thread.id");

/// Current thread name. E.g. `main`                                   
pub const THREAD_NAME: Key = Key::from_static_str("thread.name");

/// An identifier for the database management system (DBMS) product being used.
/// See [semantic conventions] page for a list of well-known identifiers.
///
/// [semantic conventions page]: https://github.com/open-telemetry/opentelemetry-specification/blob/master/specification/trace/semantic_conventions/database.md#notes-and-well-known-identifiers-for-dbsystem
pub const DB_SYSTEM: Key = Key::from_static_str("db.system");

/// Username for accessing the database, e.g., `"readonly_user"` or `"reporting_user"`
pub const DB_USER: Key = Key::from_static_str("db.user");

/// The [instance name] connecting to. This name is used to determine the port of
/// a named instance.
///
/// [instance name]: https://docs.microsoft.com/en-us/sql/connect/jdbc/building-the-connection-url?view=sql-server-ver15
pub const DB_MSSQL_INSTANCE_NAME: Key = Key::from_static_str("db.mssql.instance_name");

/// The fully-qualified class name of the [Java Database Connectivity (JDBC)]
/// driver used to connect, e.g., `"org.postgresql.Driver"` or
///` "com.microsoft.sqlserver.jdbc.SQLServerDriver"`.
///
/// [Java Database Connectivity (JDBC)]: https://docs.oracle.com/javase/8/docs/technotes/guides/jdbc/
pub const DB_JDBC_DRIVER_CLASSNAME: Key = Key::from_static_str("db.jdbc.driver_classname");

/// If no tech-specific attribute is defined in the list below, this attribute
/// is used to report the name of the database being accessed. For commands that
/// switch the database, this should be set to the target database (even if the
/// command fails).
pub const DB_NAME: Key = Key::from_static_str("db.name");

/// The database statement being executed. Note that the value may be sanitized
/// to exclude sensitive information. E.g., for `db.system="other_sql"`,
/// `"SELECT * FROM wuser_table"`; for `db.system="redis"`, `"SET mykey
/// 'WuValue'"`.
pub const DB_STATEMENT: Key = Key::from_static_str("db.statement");

/// The name of the operation being executed, e.g. the [MongoDB command name]
/// such as `findAndModify`. While it would semantically make sense to set this,
/// e.g., to an SQL keyword like `SELECT` or `INSERT`, it is *not* recommended
/// to attempt any client-side parsing of `db.statement` just to get this
/// property (the back end can do that if required).
///
/// [MongoDB command name]: https://docs.mongodb.com/manual/reference/command/#database-operations
pub const DB_OPERATION: Key = Key::from_static_str("db.operation");

/// The name of the keyspace being accessed. To be used instead of the generic
/// `db.name` attribute.
pub const DB_CASSANDRA_KEYSPACE: Key = Key::from_static_str("db.cassandra.keyspace");

/// The HBase namespace being accessed. To be used instead of the generic
/// `db.name` attribute.
pub const DB_HBASE_NAMESPACE: Key = Key::from_static_str("db.hbase.namespace");

/// The index of the database being accessed as used in the `SELECT` command,
/// provided as an integer. To be used instead of the generic `db.name` attribute.
pub const DB_REDIS_DATABASE_INDEX: Key = Key::from_static_str("db.redis.database_index");

/// The collection being accessed within the database stated in `db.name`.
pub const DB_MONGODB_COLLECTION: Key = Key::from_static_str("db.mongodb.collection");

/// The type of the exception (its fully-qualified class name, if applicable).
/// The dynamic type of the exception should be preferred over the static type
/// in languages that support it. E.g. "java.net.ConnectException", "OSError"
pub const EXCEPTION_TYPE: Key = Key::from_static_str("exception.type");

/// The exception message. E.g. `"Division by zero"`, `"Can't convert 'int'
/// object to str implicitly"`
pub const EXCEPTION_MESSAGE: Key = Key::from_static_str("exception.message");

/// A stacktrace as a string in the natural representation for the language
/// runtime. The representation is to be determined and documented by each
/// language SIG. E.g.:
///
/// ```
/// stack backtrace:
///    0: <std::sys_common::backtrace::_print::DisplayBacktrace as core::fmt::Display>::fmt
///    1: core::fmt::write
///    2: std::io::Write::write_fmt
///    3: std::panicking::default_hook::{{closure}}
///    4: std::panicking::default_hook
///    5: std::panicking::rust_panic_with_hook
///    6: std::panicking::begin_panic
///    7: test::main
///    8: std::rt::lang_start::{{closure}}
///    9: std::rt::lang_start_internal
///   10: std::rt::lang_start
///   11: main
/// ```
pub const EXCEPTION_STACKTRACE: Key = Key::from_static_str("exception.stacktrace");

/// Type of the trigger on which the function is executed.
///
/// It SHOULD be one of the following strings: "datasource", "http", "pubsub",
/// "timer", or "other".
pub const FAAS_TRIGGER: Key = Key::from_static_str("faas.trigger");

/// String containing the execution id of the function. E.g.
/// `af9d5aa4-a685-4c5f-a22b-444f80b3cc28`
pub const FAAS_EXECUTION: Key = Key::from_static_str("faas.execution");

/// A boolean indicating that the serverless function is executed for the first
/// time (aka cold start).
pub const FAAS_COLDSTART: Key = Key::from_static_str("faas.coldstart");

/// The name of the source on which the operation was performed. For example, in
/// Cloud Storage or S3 corresponds to the bucket name, and in Cosmos DB to the
/// database name.
pub const FAAS_DOCUMENT_COLLECTION: Key = Key::from_static_str("faas.document.collection");

/// Describes the type of the operation that was performed on the data.
///
/// It SHOULD be one of the following strings: "insert", "edit", "delete".
pub const FAAS_DOCUMENT_OPERATION: Key = Key::from_static_str("faas.document.operation");

/// A string containing the time when the data was accessed in the [ISO 8601]
/// format expressed in [UTC]. E.g. `"2020-01-23T13:47:06Z"`
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
/// expressed in [UTC]. E.g. `"2020-01-23T13:47:06Z"`
///
/// [ISO 8601]: https://www.iso.org/iso-8601-date-and-time-format.html
/// [UTC]: https://www.w3.org/TR/NOTE-datetime
pub const FAAS_TIME: Key = Key::from_static_str("faas.time");

/// A string containing the schedule period as [Cron Expression]. E.g. `"0/5 * * * ? *"`
///
/// [Cron Expression]: https://docs.oracle.com/cd/E12058_01/doc/doc.1014/e12030/cron_expressions.htm
pub const FAAS_CRON: Key = Key::from_static_str("faas.cron");

/// HTTP request method. E.g. `"GET"`.
pub const HTTP_METHOD: Key = Key::from_static_str("http.method");

/// Full HTTP request URL in the form
/// `scheme://host[:port]/path?query[#fragment]`. Usually the fragment is not
/// transmitted over HTTP, but if it is known, it should be included
/// nevertheless.
pub const HTTP_URL: Key = Key::from_static_str("http.url");

/// The full request target as passed in a [HTTP request line] or equivalent,
/// e.g. `"/path/12314/?q=ddds#123"`.
///
/// [HTTP request line]: https://tools.ietf.org/html/rfc7230#section-3.1.1
pub const HTTP_TARGET: Key = Key::from_static_str("http.target");

/// The value of the [HTTP host header]. When the header is empty or not
/// present, this attribute should be the same.
///
/// [HTTP host header]: https://tools.ietf.org/html/rfc7230#section-5.4
pub const HTTP_HOST: Key = Key::from_static_str("http.host");

/// The URI scheme identifying the used protocol: `"http"` or `"https"`
pub const HTTP_SCHEME: Key = Key::from_static_str("http.scheme");

/// [HTTP response status code]. E.g. `200` (integer)
///
/// [HTTP response status code]: https://tools.ietf.org/html/rfc7231#section-6
pub const HTTP_STATUS_CODE: Key = Key::from_static_str("http.status_code");

/// [HTTP reason phrase]. E.g. `"OK"`
///
/// [HTTP reason phrase]: https://tools.ietf.org/html/rfc7230#section-3.1.2
pub const HTTP_STATUS_TEXT: Key = Key::from_static_str("http.status_text");

/// Kind of HTTP protocol used: `"1.0"`, `"1.1"`, `"2"`, `"SPDY"` or `"QUIC"`.
pub const HTTP_FLAVOR: Key = Key::from_static_str("http.flavor");

/// Value of the HTTP [User-Agent] header sent by the client.
///
/// [User-Agent]: https://tools.ietf.org/html/rfc7231#section-5.5.3
pub const HTTP_USER_AGENT: Key = Key::from_static_str("http.user_agent");

/// The size of the request payload body in bytes. This is the number of bytes
/// transferred excluding headers and is often, but not always, present as the
/// [Content-Length] header. For requests using transport encoding, this should
/// be the compressed size.
///
/// [Content-Length]: https://tools.ietf.org/html/rfc7230#section-3.3.2
pub const HTTP_REQUEST_CONTENT_LENGTH: Key = Key::from_static_str("http.request_content_length");

/// The size of the uncompressed request payload body after transport decoding.
/// Not set if transport encoding not used.
pub const HTTP_REQUEST_CONTENT_LENGTH_UNCOMPRESSED: Key =
    Key::from_static_str("http.request_content_length_uncompressed");

/// The size of the response payload body in bytes. This is the number of bytes
/// transferred excluding headers and is often, but not always, present as the
/// [Content-Length] header. For requests using transport encoding, this
/// should be the compressed size.
///
/// [Content-Length]: https://tools.ietf.org/html/rfc7230#section-3.3.2
pub const HTTP_RESPONSE_CONTENT_LENGTH: Key = Key::from_static_str("http.response_content_length");

/// The size of the uncompressed response payload body after transport decoding.
/// Not set if transport encoding not used.
pub const HTTP_RESPONSE_CONTENT_LENGTH_UNCOMPRESSED: Key =
    Key::from_static_str("http.response_content_length_uncompressed");

/// The primary server name of the matched virtual host. This should be obtained
/// via configuration. If no such configuration can be obtained, this attribute
/// MUST NOT be set (`net.host.name` should be used instead).
pub const HTTP_SERVER_NAME: Key = Key::from_static_str("http.server_name");

/// The matched route (path template). E.g. `"/users/:userID?"`.
pub const HTTP_ROUTE: Key = Key::from_static_str("http.route");

/// The IP address of the original client behind all proxies, if known (e.g.
/// from [X-Forwarded-For]). Note that this is not necessarily the same as
/// `net.peer.ip`, which would identify the network-level peer, which may be a
/// proxy.
///
/// [X-Forwarded-For]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For
pub const HTTP_CLIENT_IP: Key = Key::from_static_str("http.client_ip");

/// A string identifying the messaging system such as `kafka`, `rabbitmq` or `activemq`.
pub const MESSAGING_SYSTEM: Key = Key::from_static_str("messaging.system");

/// The message destination name, e.g. `MyQueue` or `MyTopic`. This might be
/// equal to the span name but is required nevertheless.
pub const MESSAGING_DESTINATION: Key = Key::from_static_str("messaging.destination");

/// The kind of message destination: Either `queue` or `topic`.
pub const MESSAGING_DESTINATION_KIND: Key = Key::from_static_str("messaging.destination_kind");

/// The name of the transport protocol such as `AMQP` or `MQTT`.
pub const MESSAGING_PROTOCOL: Key = Key::from_static_str("messaging.protocol");

/// The version of the transport protocol such as `0.9.1`.
pub const MESSAGING_PROTOCOL_VERSION: Key = Key::from_static_str("messaging.protocol_version");

/// Connection string such as `tibjmsnaming://localhost:7222` or
/// `https://queue.amazonaws.com/80398EXAMPLE/MyQueue`.
pub const MESSAGING_URL: Key = Key::from_static_str("messaging.url");

/// A value used by the messaging system as an identifier for the message,
/// represented as a string.
pub const MESSAGING_MESSAGE_ID: Key = Key::from_static_str("messaging.message_id");

/// The conversation ID identifying the conversation to which the message
/// belongs, represented as a string. Sometimes called "Correlation ID".
pub const MESSAGING_CONVERSATION_ID: Key = Key::from_static_str("messaging.conversation_id");

/// The (uncompressed) size of the message payload in bytes. Also use this
/// attribute if it is unknown whether the compressed or uncompressed payload
/// size is reported.
pub const MESSAGING_MESSAGE_PAYLOAD_SIZE_BYTES: Key =
    Key::from_static_str("messaging.message_payload_size_bytes");

/// The compressed size of the message payload in bytes.
pub const MESSAGING_MESSAGE_PAYLOAD_COMPRESSED_SIZE_BYTES: Key =
    Key::from_static_str("messaging.message_payload_compressed_size_bytes");

/// A string identifying the kind of message consumption as defined in the
/// Operation names section above. Only `"receive"` and `"process"` are used for
/// this attribute. If the operation is `"send"`, this attribute MUST NOT be
/// set, since the operation can be inferred from the span kind in that case.
pub const MESSAGING_OPERATION: Key = Key::from_static_str("messaging.operation");

/// A string identifying the remoting system, e.g., `"grpc"`, `"java_rmi"` or
/// `"wcf"`.      
pub const RPC_SYSTEM: Key = Key::from_static_str("rpc.system");

/// The full name of the service being called, including its package name, if
/// applicable.   
pub const RPC_SERVICE: Key = Key::from_static_str("rpc.service");

/// The name of the method being called, must be equal to the $method part in
/// the span name.
pub const RPC_METHOD: Key = Key::from_static_str("rpc.method");
