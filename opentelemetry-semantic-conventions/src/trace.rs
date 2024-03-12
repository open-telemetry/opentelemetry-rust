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
//! use opentelemetry::KeyValue;
//! use opentelemetry::{global, trace::Tracer as _};
//! use opentelemetry_semantic_conventions as semconv;
//!
//! let tracer = global::tracer("my-component");
//! let _span = tracer
//!     .span_builder("span-name")
//!     .with_attributes(vec![
//!         KeyValue::new(semconv::trace::NET_PEER_NAME, "example.org"),
//!         KeyValue::new(semconv::trace::NET_PEER_PORT, 80i64),
//!     ])
//!     .start(&tracer);
//! ```

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

/// Type of the trigger which caused this function invocation.
pub const FAAS_TRIGGER: &str = "faas.trigger";

/// The [`service.name`](/docs/resource/README.md#service) of the remote service. SHOULD be equal to the actual `service.name` resource attribute of the remote service if any.
///
/// # Examples
///
/// - `AuthTokenCache`
pub const PEER_SERVICE: &str = "peer.service";

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

/// Identifies the class / type of event.
///
/// Event names are subject to the same rules as [attribute names](https://github.com/open-telemetry/opentelemetry-specification/tree/v1.26.0/specification/common/attribute-naming.md). Notably, event names are namespaced to avoid collisions and provide a clean separation of semantics for events in separate domains like browser, mobile, and kubernetes.
///
/// # Examples
///
/// - `browser.mouse.click`
/// - `device.app.lifecycle`
pub const EVENT_NAME: &str = "event.name";

/// A unique identifier for the Log Record.
///
/// If an id is provided, other log records with the same id will be considered duplicates and can be removed safely. This means, that two distinguishable log records MUST have different values.
/// The id MAY be an [Universally Unique Lexicographically Sortable Identifier (ULID)](https://github.com/ulid/spec), but other identifiers (e.g. UUID) may be used as needed.
///
/// # Examples
///
/// - `01ARZ3NDEKTSV4RRFFQ69G5FAV`
pub const LOG_RECORD_UID: &str = "log.record.uid";

/// The stream associated with the log. See below for a list of well-known values.
pub const LOG_IOSTREAM: &str = "log.iostream";

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

/// This attribute represents the state the application has transitioned into at the occurrence of the event.
///
/// The iOS lifecycle states are defined in the [UIApplicationDelegate documentation](https://developer.apple.com/documentation/uikit/uiapplicationdelegate#1656902), and from which the `OS terminology` column values are derived.
pub const IOS_STATE: &str = "ios.state";

/// This attribute represents the state the application has transitioned into at the occurrence of the event.
///
/// The Android lifecycle states are defined in [Activity lifecycle callbacks](https://developer.android.com/guide/components/activities/activity-lifecycle#lc), and from which the `OS identifiers` are derived.
pub const ANDROID_STATE: &str = "android.state";

/// The name of the connection pool; unique within the instrumented application. In case the connection pool implementation doesn&#39;t provide a name, then the [db.connection_string](/docs/database/database-spans.md#connection-level-attributes) should be used.
///
/// # Examples
///
/// - `myDataSource`
pub const POOL_NAME: &str = "pool.name";

/// The state of a connection in the pool.
///
/// # Examples
///
/// - `idle`
pub const STATE: &str = "state";

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

/// Name of the buffer pool.
///
/// Pool names are generally obtained via [BufferPoolMXBean#getName()](https://docs.oracle.com/en/java/javase/11/docs/api/java.management/java/lang/management/BufferPoolMXBean.html#getName()).
///
/// # Examples
///
/// - `mapped`
/// - `direct`
pub const JVM_BUFFER_POOL_NAME: &str = "jvm.buffer.pool.name";

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

/// The device identifier.
///
/// # Examples
///
/// - `(identifier)`
pub const SYSTEM_DEVICE: &str = "system.device";

/// The logical CPU number [0..n-1].
///
/// # Examples
///
/// - `1`
pub const SYSTEM_CPU_LOGICAL_NUMBER: &str = "system.cpu.logical_number";

/// The state of the CPU.
///
/// # Examples
///
/// - `idle`
/// - `interrupt`
pub const SYSTEM_CPU_STATE: &str = "system.cpu.state";

/// The memory state.
///
/// # Examples
///
/// - `free`
/// - `cached`
pub const SYSTEM_MEMORY_STATE: &str = "system.memory.state";

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

/// A stateless protocol MUST NOT set this attribute.
///
/// # Examples
///
/// - `close_wait`
pub const SYSTEM_NETWORK_STATE: &str = "system.network.state";

/// The process state, e.g., [Linux Process State Codes](https://man7.org/linux/man-pages/man1/ps.1.html#PROCESS_STATE_CODES).
///
/// # Examples
///
/// - `running`
pub const SYSTEM_PROCESSES_STATUS: &str = "system.processes.status";

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

/// The name of the primary Cassandra table that the operation is acting upon, including the keyspace name (if applicable).
///
/// This mirrors the db.sql.table attribute but references cassandra rather than sql. It is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if it is provided by the library being instrumented. If the operation is acting upon an anonymous table, or more than one table, this value MUST NOT be set.
///
/// # Examples
///
/// - `mytable`
pub const DB_CASSANDRA_TABLE: &str = "db.cassandra.table";

/// The connection string used to connect to the database. It is recommended to remove embedded credentials.
///
/// # Examples
///
/// - `Server=(localdb)\v11.0;Integrated Security=true;`
pub const DB_CONNECTION_STRING: &str = "db.connection_string";

/// Unique Cosmos client instance id.
///
/// # Examples
///
/// - `3ba4827d-4422-483f-b59f-85b74211c11d`
pub const DB_COSMOSDB_CLIENT_ID: &str = "db.cosmosdb.client_id";

/// Cosmos client connection mode.
pub const DB_COSMOSDB_CONNECTION_MODE: &str = "db.cosmosdb.connection_mode";

/// Cosmos DB container name.
///
/// # Examples
///
/// - `anystring`
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

/// An identifier (address, unique name, or any other identifier) of the database instance that is executing queries or mutations on the current connection. This is useful in cases where the database is running in a clustered environment and the instrumentation is able to record the node executing the query. The client may obtain this value in databases like MySQL using queries like `select @@hostname`.
///
/// # Examples
///
/// - `mysql-e26b99z.example.com`
pub const DB_INSTANCE_ID: &str = "db.instance.id";

/// The fully-qualified class name of the [Java Database Connectivity (JDBC)](https://docs.oracle.com/javase/8/docs/technotes/guides/jdbc/) driver used to connect.
///
/// # Examples
///
/// - `org.postgresql.Driver`
/// - `com.microsoft.sqlserver.jdbc.SQLServerDriver`
pub const DB_JDBC_DRIVER_CLASSNAME: &str = "db.jdbc.driver_classname";

/// The MongoDB collection being accessed within the database stated in `db.name`.
///
/// # Examples
///
/// - `customers`
/// - `products`
pub const DB_MONGODB_COLLECTION: &str = "db.mongodb.collection";

/// The Microsoft SQL Server [instance name](https://docs.microsoft.com/sql/connect/jdbc/building-the-connection-url?view=sql-server-ver15) connecting to. This name is used to determine the port of a named instance.
///
/// If setting a `db.mssql.instance_name`, `server.port` is no longer required (but still recommended if non-standard).
///
/// # Examples
///
/// - `MSSQLSERVER`
pub const DB_MSSQL_INSTANCE_NAME: &str = "db.mssql.instance_name";

/// This attribute is used to report the name of the database being accessed. For commands that switch the database, this should be set to the target database (even if the command fails).
///
/// In some SQL databases, the database name to be used is called &#34;schema name&#34;. In case there are multiple layers that could be considered for database name (e.g. Oracle instance name and schema name), the database name to be used is the more specific layer (e.g. Oracle schema name).
///
/// # Examples
///
/// - `customers`
/// - `main`
pub const DB_NAME: &str = "db.name";

/// The name of the operation being executed, e.g. the [MongoDB command name](https://docs.mongodb.com/manual/reference/command/#database-operations) such as `findAndModify`, or the SQL keyword.
///
/// When setting this to an SQL keyword, it is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if the operation name is provided by the library being instrumented. If the SQL statement has an ambiguous operation, or performs more than one operation, this value may be omitted.
///
/// # Examples
///
/// - `findAndModify`
/// - `HMSET`
/// - `SELECT`
pub const DB_OPERATION: &str = "db.operation";

/// The index of the database being accessed as used in the [`SELECT` command](https://redis.io/commands/select), provided as an integer. To be used instead of the generic `db.name` attribute.
///
/// # Examples
///
/// - `0`
/// - `1`
/// - `15`
pub const DB_REDIS_DATABASE_INDEX: &str = "db.redis.database_index";

/// The name of the primary table that the operation is acting upon, including the database name (if applicable).
///
/// It is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if it is provided by the library being instrumented. If the operation is acting upon an anonymous table, or more than one table, this value MUST NOT be set.
///
/// # Examples
///
/// - `public.users`
/// - `customers`
pub const DB_SQL_TABLE: &str = "db.sql.table";

/// The database statement being executed.
///
/// # Examples
///
/// - `SELECT * FROM wuser_table`
/// - `SET mykey "WuValue"`
pub const DB_STATEMENT: &str = "db.statement";

/// An identifier for the database management system (DBMS) product being used. See below for a list of well-known identifiers.
pub const DB_SYSTEM: &str = "db.system";

/// Username for accessing the database.
///
/// # Examples
///
/// - `readonly_user`
/// - `reporting_user`
pub const DB_USER: &str = "db.user";

/// Deprecated, use `network.protocol.name` instead.
#[deprecated]
pub const HTTP_FLAVOR: &str = "http.flavor";

/// Deprecated, use `http.request.method` instead.
///
/// # Examples
///
/// - `GET`
/// - `POST`
/// - `HEAD`
#[deprecated]
pub const HTTP_METHOD: &str = "http.method";

/// Deprecated, use `http.request.header.content-length` instead.
///
/// # Examples
///
/// - `3495`
#[deprecated]
pub const HTTP_REQUEST_CONTENT_LENGTH: &str = "http.request_content_length";

/// Deprecated, use `http.response.header.content-length` instead.
///
/// # Examples
///
/// - `3495`
#[deprecated]
pub const HTTP_RESPONSE_CONTENT_LENGTH: &str = "http.response_content_length";

/// Deprecated, use `url.scheme` instead.
///
/// # Examples
///
/// - `http`
/// - `https`
#[deprecated]
pub const HTTP_SCHEME: &str = "http.scheme";

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

/// The disk IO operation direction.
///
/// # Examples
///
/// - `read`
pub const DISK_IO_DIRECTION: &str = "disk.io.direction";

/// Describes a class of error the operation ended with.
///
/// The `error.type` SHOULD be predictable and SHOULD have low cardinality.
/// Instrumentations SHOULD document the list of errors they report.
///
/// The cardinality of `error.type` within one instrumentation library SHOULD be low.
/// Telemetry consumers that aggregate data from multiple instrumentation libraries and applications
/// should be prepared for `error.type` to have high cardinality at query time when no
/// additional filters are applied.
///
/// If the operation has completed successfully, instrumentations SHOULD NOT set `error.type`.
///
/// If a specific domain defines its own set of error identifiers (such as HTTP or gRPC status codes),
/// it&#39;s RECOMMENDED to:
///
/// * Use a domain-specific attribute
/// * Set `error.type` to capture all errors, regardless of whether they are defined within the domain-specific set or not.
///
/// # Examples
///
/// - `timeout`
/// - `java.net.UnknownHostException`
/// - `server_certificate_invalid`
/// - `500`
pub const ERROR_TYPE: &str = "error.type";

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
/// as done in the [example for recording span exceptions](#recording-an-exception).
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

/// The size of the request payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
///
/// # Examples
///
/// - `3495`
pub const HTTP_REQUEST_BODY_SIZE: &str = "http.request.body.size";

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

/// The size of the response payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://www.rfc-editor.org/rfc/rfc9110.html#field.content-length) header. For requests using transport encoding, this should be the compressed size.
///
/// # Examples
///
/// - `3495`
pub const HTTP_RESPONSE_BODY_SIZE: &str = "http.response.body.size";

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
pub const MESSAGING_CLIENT_ID: &str = "messaging.client_id";

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

/// Partition the message is sent to.
///
/// # Examples
///
/// - `2`
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

/// A string identifying the kind of messaging operation.
///
/// If a custom value is used, it MUST be of low cardinality.
pub const MESSAGING_OPERATION: &str = "messaging.operation";

/// RabbitMQ message routing key.
///
/// # Examples
///
/// - `myKey`
pub const MESSAGING_RABBITMQ_DESTINATION_ROUTING_KEY: &str =
    "messaging.rabbitmq.destination.routing_key";

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

/// An identifier for the messaging system being used. See below for a list of well-known identifiers.
pub const MESSAGING_SYSTEM: &str = "messaging.system";

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

/// Peer address of the network connection - IP address or Unix domain socket name.
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
/// - `amqp`
/// - `http`
/// - `mqtt`
pub const NETWORK_PROTOCOL_NAME: &str = "network.protocol.name";

/// Version of the protocol specified in `network.protocol.name`.
///
/// `network.protocol.version` refers to the version of the protocol used and might be different from the protocol client&#39;s version. If the HTTP client has a version of `0.27.2`, but sends HTTP version `1.1`, this attribute should be set to `1.1`.
///
/// # Examples
///
/// - `3.1.1`
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
/// - `udp`
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

/// Server domain name if available without reverse DNS lookup; otherwise, IP address or Unix domain socket name.
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
/// `url.full` SHOULD capture the absolute URL when it is available (or can be reconstructed) and SHOULD NOT be validated or modified except for sanitizing purposes.
///
/// # Examples
///
/// - `https://www.foo.bar/search?q=OpenTelemetry#SemConv`
/// - `//localhost`
pub const URL_FULL: &str = "url.full";

/// The [URI path](https://www.rfc-editor.org/rfc/rfc3986#section-3.3) component.
///
/// # Examples
///
/// - `/search`
pub const URL_PATH: &str = "url.path";

/// The [URI query](https://www.rfc-editor.org/rfc/rfc3986#section-3.4) component.
///
/// Sensitive content provided in query string SHOULD be scrubbed when instrumentations can identify it.
///
/// # Examples
///
/// - `q=OpenTelemetry`
pub const URL_QUERY: &str = "url.query";

/// The [URI scheme](https://www.rfc-editor.org/rfc/rfc3986#section-3.1) component identifying the used protocol.
///
/// # Examples
///
/// - `https`
/// - `ftp`
/// - `telnet`
pub const URL_SCHEME: &str = "url.scheme";

/// Value of the [HTTP User-Agent](https://www.rfc-editor.org/rfc/rfc9110.html#field.user-agent) header sent by the client.
///
/// # Examples
///
/// - `CERN-LineMode/2.15 libwww/2.17b3`
/// - `Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.2 Mobile/15E148 Safari/604.1`
pub const USER_AGENT_ORIGINAL: &str = "user_agent.original";

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

/// The full invoked ARN as provided on the `Context` passed to the function (`Lambda-Runtime-Invoked-Function-Arn` header on the `/runtime/invocation/next` applicable).
///
/// This may be different from `cloud.resource_id` if an alias is involved.
///
/// # Examples
///
/// - `arn:aws:lambda:us-east-1:123456:function:myfunction:myalias`
pub const AWS_LAMBDA_INVOKED_ARN: &str = "aws.lambda.invoked_arn";

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

/// Parent-child Reference type.
///
/// The causal relationship between a child Span and a parent Span.
pub const OPENTRACING_REF_TYPE: &str = "opentracing.ref_type";

/// Name of the code, either &#34;OK&#34; or &#34;ERROR&#34;. MUST NOT be set if the status code is UNSET.
pub const OTEL_STATUS_CODE: &str = "otel.status_code";

/// Description of the Status if it has a value, otherwise not set.
///
/// # Examples
///
/// - `resource not found`
pub const OTEL_STATUS_DESCRIPTION: &str = "otel.status_description";

/// The invocation ID of the current function invocation.
///
/// # Examples
///
/// - `af9d5aa4-a685-4c5f-a22b-444f80b3cc28`
pub const FAAS_INVOCATION_ID: &str = "faas.invocation_id";

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

/// A string containing the schedule period as [Cron Expression](https://docs.oracle.com/cd/E12058_01/doc/doc.1014/e12030/cron_expressions.htm).
///
/// # Examples
///
/// - `0/5 * * * ? *`
pub const FAAS_CRON: &str = "faas.cron";

/// A string containing the function invocation time in the [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) format expressed in [UTC](https://www.w3.org/TR/NOTE-datetime).
///
/// # Examples
///
/// - `2020-01-23T13:47:06Z`
pub const FAAS_TIME: &str = "faas.time";

/// A boolean that is true if the serverless function is executed for the first time (aka cold-start).
pub const FAAS_COLDSTART: &str = "faas.coldstart";

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

/// The AWS request ID as returned in the response headers `x-amz-request-id` or `x-amz-requestid`.
///
/// # Examples
///
/// - `79b9da39-b7ae-508a-a6bc-864b2829c622`
/// - `C9ER4AJX75574TDJ`
pub const AWS_REQUEST_ID: &str = "aws.request_id";

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

/// The value of the `Select` request parameter.
///
/// # Examples
///
/// - `ALL_ATTRIBUTES`
/// - `COUNT`
pub const AWS_DYNAMODB_SELECT: &str = "aws.dynamodb.select";

/// The keys in the `RequestItems` object field.
///
/// # Examples
///
/// - `Users`
/// - `Cats`
pub const AWS_DYNAMODB_TABLE_NAMES: &str = "aws.dynamodb.table_names";

/// The JSON-serialized value of each item of the `GlobalSecondaryIndexes` request field.
///
/// # Examples
///
/// - `{ "IndexName": "string", "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" }, "ProvisionedThroughput": { "ReadCapacityUnits": number, "WriteCapacityUnits": number } }`
pub const AWS_DYNAMODB_GLOBAL_SECONDARY_INDEXES: &str = "aws.dynamodb.global_secondary_indexes";

/// The JSON-serialized value of each item of the `LocalSecondaryIndexes` request field.
///
/// # Examples
///
/// - `{ "IndexArn": "string", "IndexName": "string", "IndexSizeBytes": number, "ItemCount": number, "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" } }`
pub const AWS_DYNAMODB_LOCAL_SECONDARY_INDEXES: &str = "aws.dynamodb.local_secondary_indexes";

/// The value of the `ExclusiveStartTableName` request parameter.
///
/// # Examples
///
/// - `Users`
/// - `CatsTable`
pub const AWS_DYNAMODB_EXCLUSIVE_START_TABLE: &str = "aws.dynamodb.exclusive_start_table";

/// The the number of items in the `TableNames` response parameter.
///
/// # Examples
///
/// - `20`
pub const AWS_DYNAMODB_TABLE_COUNT: &str = "aws.dynamodb.table_count";

/// The value of the `ScanIndexForward` request parameter.
pub const AWS_DYNAMODB_SCAN_FORWARD: &str = "aws.dynamodb.scan_forward";

/// The value of the `Count` response parameter.
///
/// # Examples
///
/// - `10`
pub const AWS_DYNAMODB_COUNT: &str = "aws.dynamodb.count";

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

/// The value of the `TotalSegments` request parameter.
///
/// # Examples
///
/// - `100`
pub const AWS_DYNAMODB_TOTAL_SEGMENTS: &str = "aws.dynamodb.total_segments";

/// The JSON-serialized value of each item in the `AttributeDefinitions` request field.
///
/// # Examples
///
/// - `{ "AttributeName": "string", "AttributeType": "string" }`
pub const AWS_DYNAMODB_ATTRIBUTE_DEFINITIONS: &str = "aws.dynamodb.attribute_definitions";

/// The JSON-serialized value of each item in the the `GlobalSecondaryIndexUpdates` request field.
///
/// # Examples
///
/// - `{ "Create": { "IndexName": "string", "KeySchema": [ { "AttributeName": "string", "KeyType": "string" } ], "Projection": { "NonKeyAttributes": [ "string" ], "ProjectionType": "string" }, "ProvisionedThroughput": { "ReadCapacityUnits": number, "WriteCapacityUnits": number } }`
pub const AWS_DYNAMODB_GLOBAL_SECONDARY_INDEX_UPDATES: &str =
    "aws.dynamodb.global_secondary_index_updates";

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

/// Compressed size of the message in bytes.
pub const MESSAGE_COMPRESSED_SIZE: &str = "message.compressed_size";

/// MUST be calculated as two different counters starting from `1` one for sent messages and one for received message.
///
/// This way we guarantee that the values will be consistent between different implementations.
pub const MESSAGE_ID: &str = "message.id";

/// Whether this is a received or sent message.
pub const MESSAGE_TYPE: &str = "message.type";

/// Uncompressed size of the message in bytes.
pub const MESSAGE_UNCOMPRESSED_SIZE: &str = "message.uncompressed_size";
