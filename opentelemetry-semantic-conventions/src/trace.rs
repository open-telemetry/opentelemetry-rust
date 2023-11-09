// DO NOT EDIT, this is an auto-generated file
//
// If you want to update the file:
// - Edit the template at scripts/templates/semantic_attributes.rs.j2
// - Run the script at scripts/generate-consts-from-spec.sh

//! # Trace Semantic Conventions
//!
//! The [trace semantic conventions] define a set of standardized attributes to
//! be used in `Span`s.
//!
//! [trace semantic conventions]: https://github.com/open-telemetry/semantic-conventions/tree/main/model/trace
//!
//! ## Usage
//!
//! ```
//! use opentelemetry::{global, trace::Tracer as _, trace::SpanBuilder};
//! use opentelemetry_semantic_conventions as semcov;
//!
//! let tracer = global::tracer("my-component");
//! let _span = SpanBuilder::from_name("span-name")
//!     .with_attributes(vec![
//!         semcov::trace::NET_PEER_NAME.string("example.org"),
//!         semcov::trace::NET_PEER_PORT.i64(80),
//!     ])
//!     .start(&tracer);
//! ```

use opentelemetry::Key;

/// Client address - unix domain socket name, IPv4 or IPv6 address.
///
/// When observed from the server side, and when communicating through an intermediary, `client.address` SHOULD represent client address behind any intermediaries (e.g. proxies) if it&#39;s available.
///
/// # Examples
///
/// - `/tmp/my.sock`
/// - `10.1.2.80`
pub const CLIENT_ADDRESS: Key = Key::from_static_str("client.address");

/// Client port number.
///
/// When observed from the server side, and when communicating through an intermediary, `client.port` SHOULD represent client port behind any intermediaries (e.g. proxies) if it&#39;s available.
///
/// # Examples
///
/// - `65123`
pub const CLIENT_PORT: Key = Key::from_static_str("client.port");

/// Immediate client peer address - unix domain socket name, IPv4 or IPv6 address.
///
/// # Examples
///
/// - `/tmp/my.sock`
/// - `127.0.0.1`
pub const CLIENT_SOCKET_ADDRESS: Key = Key::from_static_str("client.socket.address");

/// Immediate client peer port number.
///
/// # Examples
///
/// - `35555`
pub const CLIENT_SOCKET_PORT: Key = Key::from_static_str("client.socket.port");

/// Deprecated, use `http.request.method` instead.
///
/// # Examples
///
/// - `GET`
/// - `POST`
/// - `HEAD`
#[deprecated]
pub const HTTP_METHOD: Key = Key::from_static_str("http.method");

/// Deprecated, use `http.response.status_code` instead.
///
/// # Examples
///
/// - `200`
#[deprecated]
pub const HTTP_STATUS_CODE: Key = Key::from_static_str("http.status_code");

/// Deprecated, use `url.scheme` instead.
///
/// # Examples
///
/// - `http`
/// - `https`
#[deprecated]
pub const HTTP_SCHEME: Key = Key::from_static_str("http.scheme");

/// Deprecated, use `url.full` instead.
///
/// # Examples
///
/// - `https://www.foo.bar/search?q=OpenTelemetry#SemConv`
#[deprecated]
pub const HTTP_URL: Key = Key::from_static_str("http.url");

/// Deprecated, use `url.path` and `url.query` instead.
///
/// # Examples
///
/// - `/search?q=OpenTelemetry#SemConv`
#[deprecated]
pub const HTTP_TARGET: Key = Key::from_static_str("http.target");

/// Deprecated, use `http.request.body.size` instead.
///
/// # Examples
///
/// - `3495`
#[deprecated]
pub const HTTP_REQUEST_CONTENT_LENGTH: Key = Key::from_static_str("http.request_content_length");

/// Deprecated, use `http.response.body.size` instead.
///
/// # Examples
///
/// - `3495`
#[deprecated]
pub const HTTP_RESPONSE_CONTENT_LENGTH: Key = Key::from_static_str("http.response_content_length");

/// Deprecated, use `server.socket.domain` on client spans.
///
/// # Examples
///
/// - `/var/my.sock`
#[deprecated]
pub const NET_SOCK_PEER_NAME: Key = Key::from_static_str("net.sock.peer.name");

/// Deprecated, use `server.socket.address` on client spans and `client.socket.address` on server spans.
///
/// # Examples
///
/// - `192.168.0.1`
#[deprecated]
pub const NET_SOCK_PEER_ADDR: Key = Key::from_static_str("net.sock.peer.addr");

/// Deprecated, use `server.socket.port` on client spans and `client.socket.port` on server spans.
///
/// # Examples
///
/// - `65531`
#[deprecated]
pub const NET_SOCK_PEER_PORT: Key = Key::from_static_str("net.sock.peer.port");

/// Deprecated, use `server.address` on client spans and `client.address` on server spans.
///
/// # Examples
///
/// - `example.com`
#[deprecated]
pub const NET_PEER_NAME: Key = Key::from_static_str("net.peer.name");

/// Deprecated, use `server.port` on client spans and `client.port` on server spans.
///
/// # Examples
///
/// - `8080`
#[deprecated]
pub const NET_PEER_PORT: Key = Key::from_static_str("net.peer.port");

/// Deprecated, use `server.address`.
///
/// # Examples
///
/// - `example.com`
#[deprecated]
pub const NET_HOST_NAME: Key = Key::from_static_str("net.host.name");

/// Deprecated, use `server.port`.
///
/// # Examples
///
/// - `8080`
#[deprecated]
pub const NET_HOST_PORT: Key = Key::from_static_str("net.host.port");

/// Deprecated, use `server.socket.address`.
///
/// # Examples
///
/// - `/var/my.sock`
#[deprecated]
pub const NET_SOCK_HOST_ADDR: Key = Key::from_static_str("net.sock.host.addr");

/// Deprecated, use `server.socket.port`.
///
/// # Examples
///
/// - `8080`
#[deprecated]
pub const NET_SOCK_HOST_PORT: Key = Key::from_static_str("net.sock.host.port");

/// Deprecated, use `network.transport`.
#[deprecated]
pub const NET_TRANSPORT: Key = Key::from_static_str("net.transport");

/// Deprecated, use `network.protocol.name`.
///
/// # Examples
///
/// - `amqp`
/// - `http`
/// - `mqtt`
#[deprecated]
pub const NET_PROTOCOL_NAME: Key = Key::from_static_str("net.protocol.name");

/// Deprecated, use `network.protocol.version`.
///
/// # Examples
///
/// - `3.1.1`
#[deprecated]
pub const NET_PROTOCOL_VERSION: Key = Key::from_static_str("net.protocol.version");

/// Deprecated, use `network.transport` and `network.type`.
#[deprecated]
pub const NET_SOCK_FAMILY: Key = Key::from_static_str("net.sock.family");

/// The domain name of the destination system.
///
/// This value may be a host name, a fully qualified domain name, or another host naming format.
///
/// # Examples
///
/// - `foo.example.com`
pub const DESTINATION_DOMAIN: Key = Key::from_static_str("destination.domain");

/// Peer address, for example IP address or UNIX socket name.
///
/// # Examples
///
/// - `10.5.3.2`
pub const DESTINATION_ADDRESS: Key = Key::from_static_str("destination.address");

/// Peer port number.
///
/// # Examples
///
/// - `3389`
/// - `2888`
pub const DESTINATION_PORT: Key = Key::from_static_str("destination.port");

/// The type of the exception (its fully-qualified class name, if applicable). The dynamic type of the exception should be preferred over the static type in languages that support it.
///
/// # Examples
///
/// - `java.net.ConnectException`
/// - `OSError`
pub const EXCEPTION_TYPE: Key = Key::from_static_str("exception.type");

/// The exception message.
///
/// # Examples
///
/// - `Division by zero`
/// - `Can't convert 'int' object to str implicitly`
pub const EXCEPTION_MESSAGE: Key = Key::from_static_str("exception.message");

/// A stacktrace as a string in the natural representation for the language runtime. The representation is to be determined and documented by each language SIG.
///
/// # Examples
///
/// - `Exception in thread "main" java.lang.RuntimeException: Test exception\n at com.example.GenerateTrace.methodB(GenerateTrace.java:13)\n at com.example.GenerateTrace.methodA(GenerateTrace.java:9)\n at com.example.GenerateTrace.main(GenerateTrace.java:5)`
pub const EXCEPTION_STACKTRACE: Key = Key::from_static_str("exception.stacktrace");

/// HTTP request method.
///
/// HTTP request method value SHOULD be &#34;known&#34; to the instrumentation.
/// By default, this convention defines &#34;known&#34; methods as the ones listed in [RFC9110](https://www.rfc-editor.org/rfc/rfc9110.html#name-methods)
/// and the PATCH method defined in [RFC5789](https://www.rfc-editor.org/rfc/rfc5789.html).
///
/// If the HTTP request method is not known to instrumentation, it MUST set the `http.request.method` attribute to `_OTHER` and, except if reporting a metric, MUST
/// set the exact method received in the request line as value of the `http.request.method_original` attribute.
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
pub const HTTP_REQUEST_METHOD: Key = Key::from_static_str("http.request.method");

/// [HTTP response status code](https://tools.ietf.org/html/rfc7231#section-6).
///
/// # Examples
///
/// - `200`
pub const HTTP_RESPONSE_STATUS_CODE: Key = Key::from_static_str("http.response.status_code");

/// The matched route (path template in the format used by the respective server framework). See note below.
///
/// MUST NOT be populated when this is not supported by the HTTP server framework as the route attribute should have low-cardinality and the URI path can NOT substitute it.
/// SHOULD include the [application root](/docs/http/http-spans.md#http-server-definitions) if there is one.
///
/// # Examples
///
/// - `/users/:userID?`
/// - `{controller}/{action}/{id?}`
pub const HTTP_ROUTE: Key = Key::from_static_str("http.route");

/// The name identifies the event.
///
/// # Examples
///
/// - `click`
/// - `exception`
pub const EVENT_NAME: Key = Key::from_static_str("event.name");

/// The domain identifies the business context for the events.
///
/// Events across different domains may have same `event.name`, yet be
/// unrelated events.
pub const EVENT_DOMAIN: Key = Key::from_static_str("event.domain");

/// A unique identifier for the Log Record.
///
/// If an id is provided, other log records with the same id will be considered duplicates and can be removed safely. This means, that two distinguishable log records MUST have different values.
/// The id MAY be an [Universally Unique Lexicographically Sortable Identifier (ULID)](https://github.com/ulid/spec), but other identifiers (e.g. UUID) may be used as needed.
///
/// # Examples
///
/// - `01ARZ3NDEKTSV4RRFFQ69G5FAV`
pub const LOG_RECORD_UID: Key = Key::from_static_str("log.record.uid");

/// The stream associated with the log. See below for a list of well-known values.
pub const LOG_IOSTREAM: Key = Key::from_static_str("log.iostream");

/// The basename of the file.
///
/// # Examples
///
/// - `audit.log`
pub const LOG_FILE_NAME: Key = Key::from_static_str("log.file.name");

/// The full path to the file.
///
/// # Examples
///
/// - `/var/log/mysql/audit.log`
pub const LOG_FILE_PATH: Key = Key::from_static_str("log.file.path");

/// The basename of the file, with symlinks resolved.
///
/// # Examples
///
/// - `uuid.log`
pub const LOG_FILE_NAME_RESOLVED: Key = Key::from_static_str("log.file.name_resolved");

/// The full path to the file, with symlinks resolved.
///
/// # Examples
///
/// - `/var/lib/docker/uuid.log`
pub const LOG_FILE_PATH_RESOLVED: Key = Key::from_static_str("log.file.path_resolved");

/// The type of memory.
///
/// # Examples
///
/// - `heap`
/// - `non_heap`
pub const TYPE: Key = Key::from_static_str("type");

/// Name of the memory pool.
///
/// Pool names are generally obtained via [MemoryPoolMXBean#getName()](https://docs.oracle.com/en/java/javase/11/docs/api/java.management/java/lang/management/MemoryPoolMXBean.html#getName()).
///
/// # Examples
///
/// - `G1 Old Gen`
/// - `G1 Eden space`
/// - `G1 Survivor Space`
pub const POOL: Key = Key::from_static_str("pool");

/// Logical server hostname, matches server FQDN if available, and IP or socket address if FQDN is not known.
///
/// # Examples
///
/// - `example.com`
pub const SERVER_ADDRESS: Key = Key::from_static_str("server.address");

/// Logical server port number.
///
/// # Examples
///
/// - `80`
/// - `8080`
/// - `443`
pub const SERVER_PORT: Key = Key::from_static_str("server.port");

/// The domain name of an immediate peer.
///
/// Typically observed from the client side, and represents a proxy or other intermediary domain name.
///
/// # Examples
///
/// - `proxy.example.com`
pub const SERVER_SOCKET_DOMAIN: Key = Key::from_static_str("server.socket.domain");

/// Physical server IP address or Unix socket address. If set from the client, should simply use the socket&#39;s peer address, and not attempt to find any actual server IP (i.e., if set from client, this may represent some proxy server instead of the logical server).
///
/// # Examples
///
/// - `10.5.3.2`
pub const SERVER_SOCKET_ADDRESS: Key = Key::from_static_str("server.socket.address");

/// Physical server port.
///
/// # Examples
///
/// - `16456`
pub const SERVER_SOCKET_PORT: Key = Key::from_static_str("server.socket.port");

/// The domain name of the source system.
///
/// This value may be a host name, a fully qualified domain name, or another host naming format.
///
/// # Examples
///
/// - `foo.example.com`
pub const SOURCE_DOMAIN: Key = Key::from_static_str("source.domain");

/// Source address, for example IP address or Unix socket name.
///
/// # Examples
///
/// - `10.5.3.2`
pub const SOURCE_ADDRESS: Key = Key::from_static_str("source.address");

/// Source port number.
///
/// # Examples
///
/// - `3389`
/// - `2888`
pub const SOURCE_PORT: Key = Key::from_static_str("source.port");

/// The full invoked ARN as provided on the `Context` passed to the function (`Lambda-Runtime-Invoked-Function-Arn` header on the `/runtime/invocation/next` applicable).
///
/// This may be different from `cloud.resource_id` if an alias is involved.
///
/// # Examples
///
/// - `arn:aws:lambda:us-east-1:123456:function:myfunction:myalias`
pub const AWS_LAMBDA_INVOKED_ARN: Key = Key::from_static_str("aws.lambda.invoked_arn");

/// The [event_id](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#id) uniquely identifies the event.
///
/// # Examples
///
/// - `123e4567-e89b-12d3-a456-426614174000`
/// - `0001`
pub const CLOUDEVENTS_EVENT_ID: Key = Key::from_static_str("cloudevents.event_id");

/// The [source](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#source-1) identifies the context in which an event happened.
///
/// # Examples
///
/// - `https://github.com/cloudevents`
/// - `/cloudevents/spec/pull/123`
/// - `my-service`
pub const CLOUDEVENTS_EVENT_SOURCE: Key = Key::from_static_str("cloudevents.event_source");

/// The [version of the CloudEvents specification](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#specversion) which the event uses.
///
/// # Examples
///
/// - `1.0`
pub const CLOUDEVENTS_EVENT_SPEC_VERSION: Key =
    Key::from_static_str("cloudevents.event_spec_version");

/// The [event_type](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#type) contains a value describing the type of event related to the originating occurrence.
///
/// # Examples
///
/// - `com.github.pull_request.opened`
/// - `com.example.object.deleted.v2`
pub const CLOUDEVENTS_EVENT_TYPE: Key = Key::from_static_str("cloudevents.event_type");

/// The [subject](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md#subject) of the event in the context of the event producer (identified by source).
///
/// # Examples
///
/// - `mynewfile.jpg`
pub const CLOUDEVENTS_EVENT_SUBJECT: Key = Key::from_static_str("cloudevents.event_subject");

/// Parent-child Reference type.
///
/// The causal relationship between a child Span and a parent Span.
pub const OPENTRACING_REF_TYPE: Key = Key::from_static_str("opentracing.ref_type");

/// An identifier for the database management system (DBMS) product being used. See below for a list of well-known identifiers.
pub const DB_SYSTEM: Key = Key::from_static_str("db.system");

/// The connection string used to connect to the database. It is recommended to remove embedded credentials.
///
/// # Examples
///
/// - `Server=(localdb)\v11.0;Integrated Security=true;`
pub const DB_CONNECTION_STRING: Key = Key::from_static_str("db.connection_string");

/// Username for accessing the database.
///
/// # Examples
///
/// - `readonly_user`
/// - `reporting_user`
pub const DB_USER: Key = Key::from_static_str("db.user");

/// The fully-qualified class name of the [Java Database Connectivity (JDBC)](https://docs.oracle.com/javase/8/docs/technotes/guides/jdbc/) driver used to connect.
///
/// # Examples
///
/// - `org.postgresql.Driver`
/// - `com.microsoft.sqlserver.jdbc.SQLServerDriver`
pub const DB_JDBC_DRIVER_CLASSNAME: Key = Key::from_static_str("db.jdbc.driver_classname");

/// This attribute is used to report the name of the database being accessed. For commands that switch the database, this should be set to the target database (even if the command fails).
///
/// In some SQL databases, the database name to be used is called &#34;schema name&#34;. In case there are multiple layers that could be considered for database name (e.g. Oracle instance name and schema name), the database name to be used is the more specific layer (e.g. Oracle schema name).
///
/// # Examples
///
/// - `customers`
/// - `main`
pub const DB_NAME: Key = Key::from_static_str("db.name");

/// The database statement being executed.
///
/// # Examples
///
/// - `SELECT * FROM wuser_table`
/// - `SET mykey "WuValue"`
pub const DB_STATEMENT: Key = Key::from_static_str("db.statement");

/// The name of the operation being executed, e.g. the [MongoDB command name](https://docs.mongodb.com/manual/reference/command/#database-operations) such as `findAndModify`, or the SQL keyword.
///
/// When setting this to an SQL keyword, it is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if the operation name is provided by the library being instrumented. If the SQL statement has an ambiguous operation, or performs more than one operation, this value may be omitted.
///
/// # Examples
///
/// - `findAndModify`
/// - `HMSET`
/// - `SELECT`
pub const DB_OPERATION: Key = Key::from_static_str("db.operation");

/// The Microsoft SQL Server [instance name](https://docs.microsoft.com/en-us/sql/connect/jdbc/building-the-connection-url?view=sql-server-ver15) connecting to. This name is used to determine the port of a named instance.
///
/// If setting a `db.mssql.instance_name`, `server.port` is no longer required (but still recommended if non-standard).
///
/// # Examples
///
/// - `MSSQLSERVER`
pub const DB_MSSQL_INSTANCE_NAME: Key = Key::from_static_str("db.mssql.instance_name");

/// The fetch size used for paging, i.e. how many rows will be returned at once.
///
/// # Examples
///
/// - `5000`
pub const DB_CASSANDRA_PAGE_SIZE: Key = Key::from_static_str("db.cassandra.page_size");

/// The consistency level of the query. Based on consistency values from [CQL](https://docs.datastax.com/en/cassandra-oss/3.0/cassandra/dml/dmlConfigConsistency.html).
pub const DB_CASSANDRA_CONSISTENCY_LEVEL: Key =
    Key::from_static_str("db.cassandra.consistency_level");

/// The name of the primary table that the operation is acting upon, including the keyspace name (if applicable).
///
/// This mirrors the db.sql.table attribute but references cassandra rather than sql. It is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if it is provided by the library being instrumented. If the operation is acting upon an anonymous table, or more than one table, this value MUST NOT be set.
///
/// # Examples
///
/// - `mytable`
pub const DB_CASSANDRA_TABLE: Key = Key::from_static_str("db.cassandra.table");

/// Whether or not the query is idempotent.
pub const DB_CASSANDRA_IDEMPOTENCE: Key = Key::from_static_str("db.cassandra.idempotence");

/// The number of times a query was speculatively executed. Not set or `0` if the query was not executed speculatively.
///
/// # Examples
///
/// - `0`
/// - `2`
pub const DB_CASSANDRA_SPECULATIVE_EXECUTION_COUNT: Key =
    Key::from_static_str("db.cassandra.speculative_execution_count");

/// The ID of the coordinating node for a query.
///
/// # Examples
///
/// - `be13faa2-8574-4d71-926d-27f16cf8a7af`
pub const DB_CASSANDRA_COORDINATOR_ID: Key = Key::from_static_str("db.cassandra.coordinator.id");

/// The data center of the coordinating node for a query.
///
/// # Examples
///
/// - `us-west-2`
pub const DB_CASSANDRA_COORDINATOR_DC: Key = Key::from_static_str("db.cassandra.coordinator.dc");

/// The index of the database being accessed as used in the [`SELECT` command](https://redis.io/commands/select), provided as an integer. To be used instead of the generic `db.name` attribute.
///
/// # Examples
///
/// - `0`
/// - `1`
/// - `15`
pub const DB_REDIS_DATABASE_INDEX: Key = Key::from_static_str("db.redis.database_index");

/// The collection being accessed within the database stated in `db.name`.
///
/// # Examples
///
/// - `customers`
/// - `products`
pub const DB_MONGODB_COLLECTION: Key = Key::from_static_str("db.mongodb.collection");

/// The name of the primary table that the operation is acting upon, including the database name (if applicable).
///
/// It is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if it is provided by the library being instrumented. If the operation is acting upon an anonymous table, or more than one table, this value MUST NOT be set.
///
/// # Examples
///
/// - `public.users`
/// - `customers`
pub const DB_SQL_TABLE: Key = Key::from_static_str("db.sql.table");

/// Unique Cosmos client instance id.
///
/// # Examples
///
/// - `3ba4827d-4422-483f-b59f-85b74211c11d`
pub const DB_COSMOSDB_CLIENT_ID: Key = Key::from_static_str("db.cosmosdb.client_id");

/// CosmosDB Operation Type.
pub const DB_COSMOSDB_OPERATION_TYPE: Key = Key::from_static_str("db.cosmosdb.operation_type");

/// Cosmos client connection mode.
pub const DB_COSMOSDB_CONNECTION_MODE: Key = Key::from_static_str("db.cosmosdb.connection_mode");

/// Cosmos DB container name.
///
/// # Examples
///
/// - `anystring`
pub const DB_COSMOSDB_CONTAINER: Key = Key::from_static_str("db.cosmosdb.container");

/// Request payload size in bytes.
pub const DB_COSMOSDB_REQUEST_CONTENT_LENGTH: Key =
    Key::from_static_str("db.cosmosdb.request_content_length");

/// Cosmos DB status code.
///
/// # Examples
///
/// - `200`
/// - `201`
pub const DB_COSMOSDB_STATUS_CODE: Key = Key::from_static_str("db.cosmosdb.status_code");

/// Cosmos DB sub status code.
///
/// # Examples
///
/// - `1000`
/// - `1002`
pub const DB_COSMOSDB_SUB_STATUS_CODE: Key = Key::from_static_str("db.cosmosdb.sub_status_code");

/// RU consumed for that operation.
///
/// # Examples
///
/// - `46.18`
/// - `1.0`
pub const DB_COSMOSDB_REQUEST_CHARGE: Key = Key::from_static_str("db.cosmosdb.request_charge");

/// Name of the code, either &#34;OK&#34; or &#34;ERROR&#34;. MUST NOT be set if the status code is UNSET.
pub const OTEL_STATUS_CODE: Key = Key::from_static_str("otel.status_code");

/// Description of the Status if it has a value, otherwise not set.
///
/// # Examples
///
/// - `resource not found`
pub const OTEL_STATUS_DESCRIPTION: Key = Key::from_static_str("otel.status_description");

/// Type of the trigger which caused this function invocation.
///
/// For the server/consumer span on the incoming side,
/// `faas.trigger` MUST be set.
///
/// Clients invoking FaaS instances usually cannot set `faas.trigger`,
/// since they would typically need to look in the payload to determine
/// the event type. If clients set it, it should be the same as the
/// trigger that corresponding incoming would have (i.e., this has
/// nothing to do with the underlying transport used to make the API
/// call to invoke the lambda, which is often HTTP).
pub const FAAS_TRIGGER: Key = Key::from_static_str("faas.trigger");

/// The invocation ID of the current function invocation.
///
/// # Examples
///
/// - `af9d5aa4-a685-4c5f-a22b-444f80b3cc28`
pub const FAAS_INVOCATION_ID: Key = Key::from_static_str("faas.invocation_id");

/// The name of the source on which the triggering operation was performed. For example, in Cloud Storage or S3 corresponds to the bucket name, and in Cosmos DB to the database name.
///
/// # Examples
///
/// - `myBucketName`
/// - `myDbName`
pub const FAAS_DOCUMENT_COLLECTION: Key = Key::from_static_str("faas.document.collection");

/// Describes the type of the operation that was performed on the data.
pub const FAAS_DOCUMENT_OPERATION: Key = Key::from_static_str("faas.document.operation");

/// A string containing the time when the data was accessed in the [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) format expressed in [UTC](https://www.w3.org/TR/NOTE-datetime).
///
/// # Examples
///
/// - `2020-01-23T13:47:06Z`
pub const FAAS_DOCUMENT_TIME: Key = Key::from_static_str("faas.document.time");

/// The document name/table subjected to the operation. For example, in Cloud Storage or S3 is the name of the file, and in Cosmos DB the table name.
///
/// # Examples
///
/// - `myFile.txt`
/// - `myTableName`
pub const FAAS_DOCUMENT_NAME: Key = Key::from_static_str("faas.document.name");

/// A string containing the function invocation time in the [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) format expressed in [UTC](https://www.w3.org/TR/NOTE-datetime).
///
/// # Examples
///
/// - `2020-01-23T13:47:06Z`
pub const FAAS_TIME: Key = Key::from_static_str("faas.time");

/// A string containing the schedule period as [Cron Expression](https://docs.oracle.com/cd/E12058_01/doc/doc.1014/e12030/cron_expressions.htm).
///
/// # Examples
///
/// - `0/5 * * * ? *`
pub const FAAS_CRON: Key = Key::from_static_str("faas.cron");

/// A boolean that is true if the serverless function is executed for the first time (aka cold-start).
pub const FAAS_COLDSTART: Key = Key::from_static_str("faas.coldstart");

/// The name of the invoked function.
///
/// SHOULD be equal to the `faas.name` resource attribute of the invoked function.
///
/// # Examples
///
/// - `my-function`
pub const FAAS_INVOKED_NAME: Key = Key::from_static_str("faas.invoked_name");

/// The cloud provider of the invoked function.
///
/// SHOULD be equal to the `cloud.provider` resource attribute of the invoked function.
pub const FAAS_INVOKED_PROVIDER: Key = Key::from_static_str("faas.invoked_provider");

/// The cloud region of the invoked function.
///
/// SHOULD be equal to the `cloud.region` resource attribute of the invoked function.
///
/// # Examples
///
/// - `eu-central-1`
pub const FAAS_INVOKED_REGION: Key = Key::from_static_str("faas.invoked_region");

/// The unique identifier of the feature flag.
///
/// # Examples
///
/// - `logo-color`
pub const FEATURE_FLAG_KEY: Key = Key::from_static_str("feature_flag.key");

/// The name of the service provider that performs the flag evaluation.
///
/// # Examples
///
/// - `Flag Manager`
pub const FEATURE_FLAG_PROVIDER_NAME: Key = Key::from_static_str("feature_flag.provider_name");

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
pub const FEATURE_FLAG_VARIANT: Key = Key::from_static_str("feature_flag.variant");

/// [OSI Transport Layer](https://osi-model.com/transport-layer/) or [Inter-process Communication method](https://en.wikipedia.org/wiki/Inter-process_communication). The value SHOULD be normalized to lowercase.
///
/// # Examples
///
/// - `tcp`
/// - `udp`
pub const NETWORK_TRANSPORT: Key = Key::from_static_str("network.transport");

/// [OSI Network Layer](https://osi-model.com/network-layer/) or non-OSI equivalent. The value SHOULD be normalized to lowercase.
///
/// # Examples
///
/// - `ipv4`
/// - `ipv6`
pub const NETWORK_TYPE: Key = Key::from_static_str("network.type");

/// [OSI Application Layer](https://osi-model.com/application-layer/) or non-OSI equivalent. The value SHOULD be normalized to lowercase.
///
/// # Examples
///
/// - `amqp`
/// - `http`
/// - `mqtt`
pub const NETWORK_PROTOCOL_NAME: Key = Key::from_static_str("network.protocol.name");

/// Version of the application layer protocol used. See note below.
///
/// `network.protocol.version` refers to the version of the protocol used and might be different from the protocol client&#39;s version. If the HTTP client used has a version of `0.27.2`, but sends HTTP version `1.1`, this attribute should be set to `1.1`.
///
/// # Examples
///
/// - `3.1.1`
pub const NETWORK_PROTOCOL_VERSION: Key = Key::from_static_str("network.protocol.version");

/// The internet connection type.
///
/// # Examples
///
/// - `wifi`
pub const NETWORK_CONNECTION_TYPE: Key = Key::from_static_str("network.connection.type");

/// This describes more details regarding the connection.type. It may be the type of cell technology connection, but it could be used for describing details about a wifi connection.
///
/// # Examples
///
/// - `LTE`
pub const NETWORK_CONNECTION_SUBTYPE: Key = Key::from_static_str("network.connection.subtype");

/// The name of the mobile carrier.
///
/// # Examples
///
/// - `sprint`
pub const NETWORK_CARRIER_NAME: Key = Key::from_static_str("network.carrier.name");

/// The mobile carrier country code.
///
/// # Examples
///
/// - `310`
pub const NETWORK_CARRIER_MCC: Key = Key::from_static_str("network.carrier.mcc");

/// The mobile carrier network code.
///
/// # Examples
///
/// - `001`
pub const NETWORK_CARRIER_MNC: Key = Key::from_static_str("network.carrier.mnc");

/// The ISO 3166-1 alpha-2 2-character country code associated with the mobile carrier network.
///
/// # Examples
///
/// - `DE`
pub const NETWORK_CARRIER_ICC: Key = Key::from_static_str("network.carrier.icc");

/// The [`service.name`](/docs/resource/README.md#service) of the remote service. SHOULD be equal to the actual `service.name` resource attribute of the remote service if any.
///
/// # Examples
///
/// - `AuthTokenCache`
pub const PEER_SERVICE: Key = Key::from_static_str("peer.service");

/// Username or client_id extracted from the access token or [Authorization](https://tools.ietf.org/html/rfc7235#section-4.2) header in the inbound request from outside the system.
///
/// # Examples
///
/// - `username`
pub const ENDUSER_ID: Key = Key::from_static_str("enduser.id");

/// Actual/assumed role the client is making the request under extracted from token or application security context.
///
/// # Examples
///
/// - `admin`
pub const ENDUSER_ROLE: Key = Key::from_static_str("enduser.role");

/// Scopes or granted authorities the client currently possesses extracted from token or application security context. The value would come from the scope associated with an [OAuth 2.0 Access Token](https://tools.ietf.org/html/rfc6749#section-3.3) or an attribute value in a [SAML 2.0 Assertion](http://docs.oasis-open.org/security/saml/Post2.0/sstc-saml-tech-overview-2.0.html).
///
/// # Examples
///
/// - `read:message, write:files`
pub const ENDUSER_SCOPE: Key = Key::from_static_str("enduser.scope");

/// Current &#34;managed&#34; thread ID (as opposed to OS thread ID).
///
/// # Examples
///
/// - `42`
pub const THREAD_ID: Key = Key::from_static_str("thread.id");

/// Current thread name.
///
/// # Examples
///
/// - `main`
pub const THREAD_NAME: Key = Key::from_static_str("thread.name");

/// The method or function name, or equivalent (usually rightmost part of the code unit&#39;s name).
///
/// # Examples
///
/// - `serveRequest`
pub const CODE_FUNCTION: Key = Key::from_static_str("code.function");

/// The &#34;namespace&#34; within which `code.function` is defined. Usually the qualified class or module name, such that `code.namespace` + some separator + `code.function` form a unique identifier for the code unit.
///
/// # Examples
///
/// - `com.example.MyHttpService`
pub const CODE_NAMESPACE: Key = Key::from_static_str("code.namespace");

/// The source code file name that identifies the code unit as uniquely as possible (preferably an absolute file path).
///
/// # Examples
///
/// - `/usr/local/MyApplication/content_root/app/index.php`
pub const CODE_FILEPATH: Key = Key::from_static_str("code.filepath");

/// The line number in `code.filepath` best representing the operation. It SHOULD point within the code unit named in `code.function`.
///
/// # Examples
///
/// - `42`
pub const CODE_LINENO: Key = Key::from_static_str("code.lineno");

/// The column number in `code.filepath` best representing the operation. It SHOULD point within the code unit named in `code.function`.
///
/// # Examples
///
/// - `16`
pub const CODE_COLUMN: Key = Key::from_static_str("code.column");

/// Original HTTP method sent by the client in the request line.
///
/// # Examples
///
/// - `GeT`
/// - `ACL`
/// - `foo`
pub const HTTP_REQUEST_METHOD_ORIGINAL: Key = Key::from_static_str("http.request.method_original");

/// The size of the request payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
///
/// # Examples
///
/// - `3495`
pub const HTTP_REQUEST_BODY_SIZE: Key = Key::from_static_str("http.request.body.size");

/// The size of the response payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
///
/// # Examples
///
/// - `3495`
pub const HTTP_RESPONSE_BODY_SIZE: Key = Key::from_static_str("http.response.body.size");

/// The ordinal number of request resending attempt (for any reason, including redirects).
///
/// The resend count SHOULD be updated each time an HTTP request gets resent by the client, regardless of what was the cause of the resending (e.g. redirection, authorization failure, 503 Server Unavailable, network issues, or any other).
///
/// # Examples
///
/// - `3`
pub const HTTP_RESEND_COUNT: Key = Key::from_static_str("http.resend_count");

/// The AWS request ID as returned in the response headers `x-amz-request-id` or `x-amz-requestid`.
///
/// # Examples
///
/// - `79b9da39-b7ae-508a-a6bc-864b2829c622`
/// - `C9ER4AJX75574TDJ`
pub const AWS_REQUEST_ID: Key = Key::from_static_str("aws.request_id");

/// The keys in the `RequestItems` object field.
///
/// # Examples
///
/// - `Users`
/// - `Cats`
pub const AWS_DYNAMODB_TABLE_NAMES: Key = Key::from_static_str("aws.dynamodb.table_names");

/// The JSON-serialized value of each item in the `ConsumedCapacity` response field.
///
/// # Examples
///
/// - `{ "CapacityUnits": number, "GlobalSecondaryIndexes": { "string" : { "CapacityUnits": number, "ReadCapacityUnits": number, "WriteCapacityUnits": number } }, "LocalSecondaryIndexes": { "string" : { "CapacityUnits": number, "ReadCapacityUnits": number, "WriteCapacityUnits": number } }, "ReadCapacityUnits": number, "Table": { "CapacityUnits": number, "ReadCapacityUnits": number, "WriteCapacityUnits": number }, "TableName": "string", "WriteCapacityUnits": number }`
pub const AWS_DYNAMODB_CONSUMED_CAPACITY: Key =
    Key::from_static_str("aws.dynamodb.consumed_capacity");

/// The JSON-serialized value of the `ItemCollectionMetrics` response field.
///
/// # Examples
///
/// - `{ "string" : [ { "ItemCollectionKey": { "string" : { "B": blob, "BOOL": boolean, "BS": [ blob ], "L": [ "AttributeValue" ], "M": { "string" : "AttributeValue" }, "N": "string", "NS": [ "string" ], "NULL": boolean, "S": "string", "SS": [ "string" ] } }, "SizeEstimateRangeGB": [ number ] } ] }`
pub const AWS_DYNAMODB_ITEM_COLLECTION_METRICS: Key =
    Key::from_static_str("aws.dynamodb.item_collection_metrics");

/// The value of the `ProvisionedThroughput.ReadCapacityUnits` request parameter.
///
/// # Examples
///
/// - `1.0`
/// - `2.0`
pub const AWS_DYNAMODB_PROVISIONED_READ_CAPACITY: Key =
    Key::from_static_str("aws.dynamodb.provisioned_read_capacity");

/// The value of the `ProvisionedThroughput.WriteCapacityUnits` request parameter.
///
/// # Examples
///
/// - `1.0`
/// - `2.0`
pub const AWS_DYNAMODB_PROVISIONED_WRITE_CAPACITY: Key =
    Key::from_static_str("aws.dynamodb.provisioned_write_capacity");

/// The value of the `ConsistentRead` request parameter.
pub const AWS_DYNAMODB_CONSISTENT_READ: Key = Key::from_static_str("aws.dynamodb.consistent_read");

/// The value of the `ProjectionExpression` request parameter.
///
/// # Examples
///
/// - `Title`
/// - `Title, Price, Color`
/// - `Title, Description, RelatedItems, ProductReviews`
pub const AWS_DYNAMODB_PROJECTION: Key = Key::from_static_str("aws.dynamodb.projection");

/// The value of the `Limit` request parameter.
///
/// # Examples
///
/// - `10`
pub const AWS_DYNAMODB_LIMIT: Key = Key::from_static_str("aws.dynamodb.limit");

/// The value of the `AttributesToGet` request parameter.
///
/// # Examples
///
/// - `lives`
/// - `id`
pub const AWS_DYNAMODB_ATTRIBUTES_TO_GET: Key =
    Key::from_static_str("aws.dynamodb.attributes_to_get");

/// The value of the `IndexName` request parameter.
///
/// # Examples
///
/// - `name_to_group`
pub const AWS_DYNAMODB_INDEX_NAME: Key = Key::from_static_str("aws.dynamodb.index_name");

/// The value of the `Select` request parameter.
///
/// # Examples
///
/// - `ALL_ATTRIBUTES`
/// - `COUNT`
pub const AWS_DYNAMODB_SELECT: Key = Key::from_static_str("aws.dynamodb.select");

/// The JSON-serialized value of each item of the `GlobalSecondaryIndexes` request field.
///
/// # Examples
///
/// - `{ "IndexName": "string", "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" }, "ProvisionedThroughput": { "ReadCapacityUnits": number, "WriteCapacityUnits": number } }`
pub const AWS_DYNAMODB_GLOBAL_SECONDARY_INDEXES: Key =
    Key::from_static_str("aws.dynamodb.global_secondary_indexes");

/// The JSON-serialized value of each item of the `LocalSecondaryIndexes` request field.
///
/// # Examples
///
/// - `{ "IndexArn": "string", "IndexName": "string", "IndexSizeBytes": number, "ItemCount": number, "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" } }`
pub const AWS_DYNAMODB_LOCAL_SECONDARY_INDEXES: Key =
    Key::from_static_str("aws.dynamodb.local_secondary_indexes");

/// The value of the `ExclusiveStartTableName` request parameter.
///
/// # Examples
///
/// - `Users`
/// - `CatsTable`
pub const AWS_DYNAMODB_EXCLUSIVE_START_TABLE: Key =
    Key::from_static_str("aws.dynamodb.exclusive_start_table");

/// The the number of items in the `TableNames` response parameter.
///
/// # Examples
///
/// - `20`
pub const AWS_DYNAMODB_TABLE_COUNT: Key = Key::from_static_str("aws.dynamodb.table_count");

/// The value of the `ScanIndexForward` request parameter.
pub const AWS_DYNAMODB_SCAN_FORWARD: Key = Key::from_static_str("aws.dynamodb.scan_forward");

/// The value of the `Segment` request parameter.
///
/// # Examples
///
/// - `10`
pub const AWS_DYNAMODB_SEGMENT: Key = Key::from_static_str("aws.dynamodb.segment");

/// The value of the `TotalSegments` request parameter.
///
/// # Examples
///
/// - `100`
pub const AWS_DYNAMODB_TOTAL_SEGMENTS: Key = Key::from_static_str("aws.dynamodb.total_segments");

/// The value of the `Count` response parameter.
///
/// # Examples
///
/// - `10`
pub const AWS_DYNAMODB_COUNT: Key = Key::from_static_str("aws.dynamodb.count");

/// The value of the `ScannedCount` response parameter.
///
/// # Examples
///
/// - `50`
pub const AWS_DYNAMODB_SCANNED_COUNT: Key = Key::from_static_str("aws.dynamodb.scanned_count");

/// The JSON-serialized value of each item in the `AttributeDefinitions` request field.
///
/// # Examples
///
/// - `{ "AttributeName": "string", "AttributeType": "string" }`
pub const AWS_DYNAMODB_ATTRIBUTE_DEFINITIONS: Key =
    Key::from_static_str("aws.dynamodb.attribute_definitions");

/// The JSON-serialized value of each item in the the `GlobalSecondaryIndexUpdates` request field.
///
/// # Examples
///
/// - `{ "Create": { "IndexName": "string", "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" }, "ProvisionedThroughput": { "ReadCapacityUnits": number, "WriteCapacityUnits": number } }`
pub const AWS_DYNAMODB_GLOBAL_SECONDARY_INDEX_UPDATES: Key =
    Key::from_static_str("aws.dynamodb.global_secondary_index_updates");

/// The S3 bucket name the request refers to. Corresponds to the `--bucket` parameter of the [S3 API](https://docs.aws.amazon.com/cli/latest/reference/s3api/index.html) operations.
///
/// The `bucket` attribute is applicable to all S3 operations that reference a bucket, i.e. that require the bucket name as a mandatory parameter.
/// This applies to almost all S3 operations except `list-buckets`.
///
/// # Examples
///
/// - `some-bucket-name`
pub const AWS_S3_BUCKET: Key = Key::from_static_str("aws.s3.bucket");

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
pub const AWS_S3_KEY: Key = Key::from_static_str("aws.s3.key");

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
pub const AWS_S3_COPY_SOURCE: Key = Key::from_static_str("aws.s3.copy_source");

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
pub const AWS_S3_UPLOAD_ID: Key = Key::from_static_str("aws.s3.upload_id");

/// The delete request container that specifies the objects to be deleted.
///
/// The `delete` attribute is only applicable to the [delete-object](https://docs.aws.amazon.com/cli/latest/reference/s3api/delete-object.html) operation.
/// The `delete` attribute corresponds to the `--delete` parameter of the
/// [delete-objects operation within the S3 API](https://docs.aws.amazon.com/cli/latest/reference/s3api/delete-objects.html).
///
/// # Examples
///
/// - `Objects=[{Key=string,VersionId=string},{Key=string,VersionId=string}],Quiet=boolean`
pub const AWS_S3_DELETE: Key = Key::from_static_str("aws.s3.delete");

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
pub const AWS_S3_PART_NUMBER: Key = Key::from_static_str("aws.s3.part_number");

/// The name of the operation being executed.
///
/// # Examples
///
/// - `findBookById`
pub const GRAPHQL_OPERATION_NAME: Key = Key::from_static_str("graphql.operation.name");

/// The type of the operation being executed.
///
/// # Examples
///
/// - `query`
/// - `mutation`
/// - `subscription`
pub const GRAPHQL_OPERATION_TYPE: Key = Key::from_static_str("graphql.operation.type");

/// The GraphQL document being executed.
///
/// The value may be sanitized to exclude sensitive information.
///
/// # Examples
///
/// - `query findBookById { bookById(id: ?) { name } }`
pub const GRAPHQL_DOCUMENT: Key = Key::from_static_str("graphql.document");

/// A value used by the messaging system as an identifier for the message, represented as a string.
///
/// # Examples
///
/// - `452a7c7c7c7048c2f887f61572b18fc2`
pub const MESSAGING_MESSAGE_ID: Key = Key::from_static_str("messaging.message.id");

/// The [conversation ID](#conversations) identifying the conversation to which the message belongs, represented as a string. Sometimes called &#34;Correlation ID&#34;.
///
/// # Examples
///
/// - `MyConversationId`
pub const MESSAGING_MESSAGE_CONVERSATION_ID: Key =
    Key::from_static_str("messaging.message.conversation_id");

/// The (uncompressed) size of the message payload in bytes. Also use this attribute if it is unknown whether the compressed or uncompressed payload size is reported.
///
/// # Examples
///
/// - `2738`
pub const MESSAGING_MESSAGE_PAYLOAD_SIZE_BYTES: Key =
    Key::from_static_str("messaging.message.payload_size_bytes");

/// The compressed size of the message payload in bytes.
///
/// # Examples
///
/// - `2048`
pub const MESSAGING_MESSAGE_PAYLOAD_COMPRESSED_SIZE_BYTES: Key =
    Key::from_static_str("messaging.message.payload_compressed_size_bytes");

/// The message destination name.
///
/// Destination name SHOULD uniquely identify a specific queue, topic or other entity within the broker. If
/// the broker does not have such notion, the destination name SHOULD uniquely identify the broker.
///
/// # Examples
///
/// - `MyQueue`
/// - `MyTopic`
pub const MESSAGING_DESTINATION_NAME: Key = Key::from_static_str("messaging.destination.name");

/// Low cardinality representation of the messaging destination name.
///
/// Destination names could be constructed from templates. An example would be a destination name involving a user name or product id. Although the destination name in this case is of high cardinality, the underlying template is of low cardinality and can be effectively used for grouping and aggregation.
///
/// # Examples
///
/// - `/customers/{customerId}`
pub const MESSAGING_DESTINATION_TEMPLATE: Key =
    Key::from_static_str("messaging.destination.template");

/// A boolean that is true if the message destination is temporary and might not exist anymore after messages are processed.
pub const MESSAGING_DESTINATION_TEMPORARY: Key =
    Key::from_static_str("messaging.destination.temporary");

/// A boolean that is true if the message destination is anonymous (could be unnamed or have auto-generated name).
pub const MESSAGING_DESTINATION_ANONYMOUS: Key =
    Key::from_static_str("messaging.destination.anonymous");

/// A string identifying the messaging system.
///
/// # Examples
///
/// - `kafka`
/// - `rabbitmq`
/// - `rocketmq`
/// - `activemq`
/// - `AmazonSQS`
pub const MESSAGING_SYSTEM: Key = Key::from_static_str("messaging.system");

/// A string identifying the kind of messaging operation as defined in the [Operation names](#operation-names) section above.
///
/// If a custom value is used, it MUST be of low cardinality.
pub const MESSAGING_OPERATION: Key = Key::from_static_str("messaging.operation");

/// The number of messages sent, received, or processed in the scope of the batching operation.
///
/// Instrumentations SHOULD NOT set `messaging.batch.message_count` on spans that operate with a single message. When a messaging client library supports both batch and single-message API for the same operation, instrumentations SHOULD use `messaging.batch.message_count` for batching APIs and SHOULD NOT use it for single-message APIs.
///
/// # Examples
///
/// - `0`
/// - `1`
/// - `2`
pub const MESSAGING_BATCH_MESSAGE_COUNT: Key =
    Key::from_static_str("messaging.batch.message_count");

/// A unique identifier for the client that consumes or produces a message.
///
/// # Examples
///
/// - `client-5`
/// - `myhost@8742@s8083jm`
pub const MESSAGING_CLIENT_ID: Key = Key::from_static_str("messaging.client_id");

/// RabbitMQ message routing key.
///
/// # Examples
///
/// - `myKey`
pub const MESSAGING_RABBITMQ_DESTINATION_ROUTING_KEY: Key =
    Key::from_static_str("messaging.rabbitmq.destination.routing_key");

/// Message keys in Kafka are used for grouping alike messages to ensure they&#39;re processed on the same partition. They differ from `messaging.message.id` in that they&#39;re not unique. If the key is `null`, the attribute MUST NOT be set.
///
/// If the key type is not string, it&#39;s string representation has to be supplied for the attribute. If the key has no unambiguous, canonical string form, don&#39;t include its value.
///
/// # Examples
///
/// - `myKey`
pub const MESSAGING_KAFKA_MESSAGE_KEY: Key = Key::from_static_str("messaging.kafka.message.key");

/// Name of the Kafka Consumer Group that is handling the message. Only applies to consumers, not producers.
///
/// # Examples
///
/// - `my-group`
pub const MESSAGING_KAFKA_CONSUMER_GROUP: Key =
    Key::from_static_str("messaging.kafka.consumer.group");

/// Partition the message is sent to.
///
/// # Examples
///
/// - `2`
pub const MESSAGING_KAFKA_DESTINATION_PARTITION: Key =
    Key::from_static_str("messaging.kafka.destination.partition");

/// The offset of a record in the corresponding Kafka partition.
///
/// # Examples
///
/// - `42`
pub const MESSAGING_KAFKA_MESSAGE_OFFSET: Key =
    Key::from_static_str("messaging.kafka.message.offset");

/// A boolean that is true if the message is a tombstone.
pub const MESSAGING_KAFKA_MESSAGE_TOMBSTONE: Key =
    Key::from_static_str("messaging.kafka.message.tombstone");

/// Namespace of RocketMQ resources, resources in different namespaces are individual.
///
/// # Examples
///
/// - `myNamespace`
pub const MESSAGING_ROCKETMQ_NAMESPACE: Key = Key::from_static_str("messaging.rocketmq.namespace");

/// Name of the RocketMQ producer/consumer group that is handling the message. The client type is identified by the SpanKind.
///
/// # Examples
///
/// - `myConsumerGroup`
pub const MESSAGING_ROCKETMQ_CLIENT_GROUP: Key =
    Key::from_static_str("messaging.rocketmq.client_group");

/// The timestamp in milliseconds that the delay message is expected to be delivered to consumer.
///
/// # Examples
///
/// - `1665987217045`
pub const MESSAGING_ROCKETMQ_MESSAGE_DELIVERY_TIMESTAMP: Key =
    Key::from_static_str("messaging.rocketmq.message.delivery_timestamp");

/// The delay time level for delay message, which determines the message delay time.
///
/// # Examples
///
/// - `3`
pub const MESSAGING_ROCKETMQ_MESSAGE_DELAY_TIME_LEVEL: Key =
    Key::from_static_str("messaging.rocketmq.message.delay_time_level");

/// It is essential for FIFO message. Messages that belong to the same message group are always processed one by one within the same consumer group.
///
/// # Examples
///
/// - `myMessageGroup`
pub const MESSAGING_ROCKETMQ_MESSAGE_GROUP: Key =
    Key::from_static_str("messaging.rocketmq.message.group");

/// Type of message.
pub const MESSAGING_ROCKETMQ_MESSAGE_TYPE: Key =
    Key::from_static_str("messaging.rocketmq.message.type");

/// The secondary classifier of message besides topic.
///
/// # Examples
///
/// - `tagA`
pub const MESSAGING_ROCKETMQ_MESSAGE_TAG: Key =
    Key::from_static_str("messaging.rocketmq.message.tag");

/// Key(s) of message, another way to mark message besides message id.
///
/// # Examples
///
/// - `keyA`
/// - `keyB`
pub const MESSAGING_ROCKETMQ_MESSAGE_KEYS: Key =
    Key::from_static_str("messaging.rocketmq.message.keys");

/// Model of message consumption. This only applies to consumer spans.
pub const MESSAGING_ROCKETMQ_CONSUMPTION_MODEL: Key =
    Key::from_static_str("messaging.rocketmq.consumption_model");

/// A string identifying the remoting system. See below for a list of well-known identifiers.
pub const RPC_SYSTEM: Key = Key::from_static_str("rpc.system");

/// The full (logical) name of the service being called, including its package name, if applicable.
///
/// This is the logical name of the service from the RPC interface perspective, which can be different from the name of any implementing class. The `code.namespace` attribute may be used to store the latter (despite the attribute name, it may include a class name; e.g., class with method actually executing the call on the server side, RPC client stub class on the client side).
///
/// # Examples
///
/// - `myservice.EchoService`
pub const RPC_SERVICE: Key = Key::from_static_str("rpc.service");

/// The name of the (logical) method being called, must be equal to the $method part in the span name.
///
/// This is the logical name of the method from the RPC interface perspective, which can be different from the name of any implementing method/function. The `code.function` attribute may be used to store the latter (e.g., method actually executing the call on the server side, RPC client stub method on the client side).
///
/// # Examples
///
/// - `exampleMethod`
pub const RPC_METHOD: Key = Key::from_static_str("rpc.method");

/// The [numeric status code](https://github.com/grpc/grpc/blob/v1.33.2/doc/statuscodes.md) of the gRPC request.
pub const RPC_GRPC_STATUS_CODE: Key = Key::from_static_str("rpc.grpc.status_code");

/// Protocol version as in `jsonrpc` property of request/response. Since JSON-RPC 1.0 does not specify this, the value can be omitted.
///
/// # Examples
///
/// - `2.0`
/// - `1.0`
pub const RPC_JSONRPC_VERSION: Key = Key::from_static_str("rpc.jsonrpc.version");

/// `id` property of request or response. Since protocol allows id to be int, string, `null` or missing (for notifications), value is expected to be cast to string for simplicity. Use empty string in case of `null` value. Omit entirely if this is a notification.
///
/// # Examples
///
/// - `10`
/// - `request-7`
/// - ``
pub const RPC_JSONRPC_REQUEST_ID: Key = Key::from_static_str("rpc.jsonrpc.request_id");

/// `error.code` property of response if it is an error response.
///
/// # Examples
///
/// - `-32700`
/// - `100`
pub const RPC_JSONRPC_ERROR_CODE: Key = Key::from_static_str("rpc.jsonrpc.error_code");

/// `error.message` property of response if it is an error response.
///
/// # Examples
///
/// - `Parse error`
/// - `User already exists`
pub const RPC_JSONRPC_ERROR_MESSAGE: Key = Key::from_static_str("rpc.jsonrpc.error_message");

/// Whether this is a received or sent message.
pub const MESSAGE_TYPE: Key = Key::from_static_str("message.type");

/// MUST be calculated as two different counters starting from `1` one for sent messages and one for received message.
///
/// This way we guarantee that the values will be consistent between different implementations.
pub const MESSAGE_ID: Key = Key::from_static_str("message.id");

/// Compressed size of the message in bytes.
pub const MESSAGE_COMPRESSED_SIZE: Key = Key::from_static_str("message.compressed_size");

/// Uncompressed size of the message in bytes.
pub const MESSAGE_UNCOMPRESSED_SIZE: Key = Key::from_static_str("message.uncompressed_size");

/// The [error codes](https://connect.build/docs/protocol/#error-codes) of the Connect request. Error codes are always string values.
pub const RPC_CONNECT_RPC_ERROR_CODE: Key = Key::from_static_str("rpc.connect_rpc.error_code");

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
/// as done in the [example above](#recording-an-exception).
///
/// It follows that an exception may still escape the scope of the span
/// even if the `exception.escaped` attribute was not set or set to false,
/// since the event might have been recorded at a time where it was not
/// clear whether the exception will escape.
pub const EXCEPTION_ESCAPED: Key = Key::from_static_str("exception.escaped");

/// The [URI scheme](https://www.rfc-editor.org/rfc/rfc3986#section-3.1) component identifying the used protocol.
///
/// # Examples
///
/// - `https`
/// - `ftp`
/// - `telnet`
pub const URL_SCHEME: Key = Key::from_static_str("url.scheme");

/// Absolute URL describing a network resource according to [RFC3986](https://www.rfc-editor.org/rfc/rfc3986).
///
/// For network calls, URL usually has `scheme://host[:port][path][?query][#fragment]` format, where the fragment is not transmitted over HTTP, but if it is known, it should be included nevertheless.
/// `url.full` MUST NOT contain credentials passed via URL in form of `https://username:password@www.example.com/`. In such case username and password should be redacted and attribute&#39;s value should be `https://REDACTED:REDACTED@www.example.com/`.
/// `url.full` SHOULD capture the absolute URL when it is available (or can be reconstructed) and SHOULD NOT be validated or modified except for sanitizing purposes.
///
/// # Examples
///
/// - `https://www.foo.bar/search?q=OpenTelemetry#SemConv`
/// - `//localhost`
pub const URL_FULL: Key = Key::from_static_str("url.full");

/// The [URI path](https://www.rfc-editor.org/rfc/rfc3986#section-3.3) component.
///
/// When missing, the value is assumed to be `/`
///
/// # Examples
///
/// - `/search`
pub const URL_PATH: Key = Key::from_static_str("url.path");

/// The [URI query](https://www.rfc-editor.org/rfc/rfc3986#section-3.4) component.
///
/// Sensitive content provided in query string SHOULD be scrubbed when instrumentations can identify it.
///
/// # Examples
///
/// - `q=OpenTelemetry`
pub const URL_QUERY: Key = Key::from_static_str("url.query");

/// The [URI fragment](https://www.rfc-editor.org/rfc/rfc3986#section-3.5) component.
///
/// # Examples
///
/// - `SemConv`
pub const URL_FRAGMENT: Key = Key::from_static_str("url.fragment");

/// Value of the [HTTP User-Agent](https://www.rfc-editor.org/rfc/rfc9110.html#field.user-agent) header sent by the client.
///
/// # Examples
///
/// - `CERN-LineMode/2.15 libwww/2.17b3`
pub const USER_AGENT_ORIGINAL: Key = Key::from_static_str("user_agent.original");
