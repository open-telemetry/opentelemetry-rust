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
//! [trace semantic conventions]: https://github.com/open-telemetry/opentelemetry-specification/tree/master/specification/trace/semantic_conventions
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::{global, trace::Tracer as _};
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

/// An identifier for the database management system (DBMS) product being used. See below for a list of well-known identifiers.
pub const DB_SYSTEM: Key = Key::from_static_str("db.system");

/// The connection string used to connect to the database. It is recommended to remove embedded credentials.
///
/// # Examples
///
/// - Server=(localdb)\v11.0;Integrated Security=true;
pub const DB_CONNECTION_STRING: Key = Key::from_static_str("db.connection_string");

/// Username for accessing the database.
///
/// # Examples
///
/// - readonly_user
/// - reporting_user
pub const DB_USER: Key = Key::from_static_str("db.user");

/// The fully-qualified class name of the [Java Database Connectivity (JDBC)](https://docs.oracle.com/javase/8/docs/technotes/guides/jdbc/) driver used to connect.
///
/// # Examples
///
/// - org.postgresql.Driver
/// - com.microsoft.sqlserver.jdbc.SQLServerDriver
pub const DB_JDBC_DRIVER_CLASSNAME: Key = Key::from_static_str("db.jdbc.driver_classname");

/// If no [tech-specific attribute](#call-level-attributes-for-specific-technologies) is defined, this attribute is used to report the name of the database being accessed. For commands that switch the database, this should be set to the target database (even if the command fails).
///
/// In some SQL databases, the database name to be used is called &#34;schema name&#34;.
///
/// # Examples
///
/// - customers
/// - main
pub const DB_NAME: Key = Key::from_static_str("db.name");

/// The database statement being executed.
///
/// The value may be sanitized to exclude sensitive information.
///
/// # Examples
///
/// - SELECT * FROM wuser_table
/// - SET mykey &#34;WuValue&#34;
pub const DB_STATEMENT: Key = Key::from_static_str("db.statement");

/// The name of the operation being executed, e.g. the [MongoDB command name](https://docs.mongodb.com/manual/reference/command/#database-operations) such as `findAndModify`, or the SQL keyword.
///
/// When setting this to an SQL keyword, it is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if the operation name is provided by the library being instrumented. If the SQL statement has an ambiguous operation, or performs more than one operation, this value may be omitted.
///
/// # Examples
///
/// - findAndModify
/// - HMSET
/// - SELECT
pub const DB_OPERATION: Key = Key::from_static_str("db.operation");

/// Remote hostname or similar, see note below.
///
/// # Examples
///
/// - example.com
pub const NET_PEER_NAME: Key = Key::from_static_str("net.peer.name");

/// Remote address of the peer (dotted decimal for IPv4 or [RFC5952](https://tools.ietf.org/html/rfc5952) for IPv6).
///
/// # Examples
///
/// - 127.0.0.1
pub const NET_PEER_IP: Key = Key::from_static_str("net.peer.ip");

/// Remote port number.
///
/// # Examples
///
/// - 80
/// - 8080
/// - 443
pub const NET_PEER_PORT: Key = Key::from_static_str("net.peer.port");

/// Transport protocol used. See note below.
///
/// # Examples
///
/// - IP.TCP
pub const NET_TRANSPORT: Key = Key::from_static_str("net.transport");

/// The Microsoft SQL Server [instance name](https://docs.microsoft.com/en-us/sql/connect/jdbc/building-the-connection-url?view=sql-server-ver15) connecting to. This name is used to determine the port of a named instance.
///
/// If setting a `db.mssql.instance_name`, `net.peer.port` is no longer required (but still recommended if non-standard).
///
/// # Examples
///
/// - MSSQLSERVER
pub const DB_MSSQL_INSTANCE_NAME: Key = Key::from_static_str("db.mssql.instance_name");

/// The name of the keyspace being accessed. To be used instead of the generic `db.name` attribute.
///
/// # Examples
///
/// - mykeyspace
pub const DB_CASSANDRA_KEYSPACE: Key = Key::from_static_str("db.cassandra.keyspace");

/// The fetch size used for paging, i.e. how many rows will be returned at once.
///
/// # Examples
///
/// - 5000
pub const DB_CASSANDRA_PAGE_SIZE: Key = Key::from_static_str("db.cassandra.page_size");

/// The consistency level of the query. Based on consistency values from [CQL](https://docs.datastax.com/en/cassandra-oss/3.0/cassandra/dml/dmlConfigConsistency.html).
pub const DB_CASSANDRA_CONSISTENCY_LEVEL: Key =
    Key::from_static_str("db.cassandra.consistency_level");

/// The name of the primary table that the operation is acting upon, including the schema name (if applicable).
///
/// This mirrors the db.sql.table attribute but references cassandra rather than sql. It is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if it is provided by the library being instrumented. If the operation is acting upon an anonymous table, or more than one table, this value MUST NOT be set.
///
/// # Examples
///
/// - mytable
pub const DB_CASSANDRA_TABLE: Key = Key::from_static_str("db.cassandra.table");

/// Whether or not the query is idempotent.
pub const DB_CASSANDRA_IDEMPOTENCE: Key = Key::from_static_str("db.cassandra.idempotence");

/// The number of times a query was speculatively executed. Not set or `0` if the query was not executed speculatively.
///
/// # Examples
///
/// - 0
/// - 2
pub const DB_CASSANDRA_SPECULATIVE_EXECUTION_COUNT: Key =
    Key::from_static_str("db.cassandra.speculative_execution_count");

/// The ID of the coordinating node for a query.
///
/// # Examples
///
/// - be13faa2-8574-4d71-926d-27f16cf8a7af
pub const DB_CASSANDRA_COORDINATOR_ID: Key = Key::from_static_str("db.cassandra.coordinator.id");

/// The data center of the coordinating node for a query.
///
/// # Examples
///
/// - us-west-2
pub const DB_CASSANDRA_COORDINATOR_DC: Key = Key::from_static_str("db.cassandra.coordinator.dc");

/// The [HBase namespace](https://hbase.apache.org/book.html#_namespace) being accessed. To be used instead of the generic `db.name` attribute.
///
/// # Examples
///
/// - default
pub const DB_HBASE_NAMESPACE: Key = Key::from_static_str("db.hbase.namespace");

/// The index of the database being accessed as used in the [`SELECT` command](https://redis.io/commands/select), provided as an integer. To be used instead of the generic `db.name` attribute.
///
/// # Examples
///
/// - 0
/// - 1
/// - 15
pub const DB_REDIS_DATABASE_INDEX: Key = Key::from_static_str("db.redis.database_index");

/// The collection being accessed within the database stated in `db.name`.
///
/// # Examples
///
/// - customers
/// - products
pub const DB_MONGODB_COLLECTION: Key = Key::from_static_str("db.mongodb.collection");

/// The name of the primary table that the operation is acting upon, including the schema name (if applicable).
///
/// It is not recommended to attempt any client-side parsing of `db.statement` just to get this property, but it should be set if it is provided by the library being instrumented. If the operation is acting upon an anonymous table, or more than one table, this value MUST NOT be set.
///
/// # Examples
///
/// - public.users
/// - customers
pub const DB_SQL_TABLE: Key = Key::from_static_str("db.sql.table");

/// The type of the exception (its fully-qualified class name, if applicable). The dynamic type of the exception should be preferred over the static type in languages that support it.
///
/// # Examples
///
/// - java.net.ConnectException
/// - OSError
pub const EXCEPTION_TYPE: Key = Key::from_static_str("exception.type");

/// The exception message.
///
/// # Examples
///
/// - Division by zero
/// - Can&#39;t convert &#39;int&#39; object to str implicitly
pub const EXCEPTION_MESSAGE: Key = Key::from_static_str("exception.message");

/// A stacktrace as a string in the natural representation for the language runtime. The representation is to be determined and documented by each language SIG.
///
/// # Examples
///
/// - Exception in thread &#34;main&#34; java.lang.RuntimeException: Test exception\n at com.example.GenerateTrace.methodB(GenerateTrace.java:13)\n at com.example.GenerateTrace.methodA(GenerateTrace.java:9)\n at com.example.GenerateTrace.main(GenerateTrace.java:5)
pub const EXCEPTION_STACKTRACE: Key = Key::from_static_str("exception.stacktrace");

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
/// as done in the [example above](#exception-end-example).
///
/// It follows that an exception may still escape the scope of the span
/// even if the `exception.escaped` attribute was not set or set to false,
/// since the event might have been recorded at a time where it was not
/// clear whether the exception will escape.
pub const EXCEPTION_ESCAPED: Key = Key::from_static_str("exception.escaped");

/// Type of the trigger on which the function is executed.
pub const FAAS_TRIGGER: Key = Key::from_static_str("faas.trigger");

/// The execution ID of the current function execution.
///
/// # Examples
///
/// - af9d5aa4-a685-4c5f-a22b-444f80b3cc28
pub const FAAS_EXECUTION: Key = Key::from_static_str("faas.execution");

/// The name of the source on which the triggering operation was performed. For example, in Cloud Storage or S3 corresponds to the bucket name, and in Cosmos DB to the database name.
///
/// # Examples
///
/// - myBucketName
/// - myDbName
pub const FAAS_DOCUMENT_COLLECTION: Key = Key::from_static_str("faas.document.collection");

/// Describes the type of the operation that was performed on the data.
pub const FAAS_DOCUMENT_OPERATION: Key = Key::from_static_str("faas.document.operation");

/// A string containing the time when the data was accessed in the [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) format expressed in [UTC](https://www.w3.org/TR/NOTE-datetime).
///
/// # Examples
///
/// - 2020-01-23T13:47:06Z
pub const FAAS_DOCUMENT_TIME: Key = Key::from_static_str("faas.document.time");

/// The document name/table subjected to the operation. For example, in Cloud Storage or S3 is the name of the file, and in Cosmos DB the table name.
///
/// # Examples
///
/// - myFile.txt
/// - myTableName
pub const FAAS_DOCUMENT_NAME: Key = Key::from_static_str("faas.document.name");

/// HTTP request method.
///
/// # Examples
///
/// - GET
/// - POST
/// - HEAD
pub const HTTP_METHOD: Key = Key::from_static_str("http.method");

/// Full HTTP request URL in the form `scheme://host[:port]/path?query[#fragment]`. Usually the fragment is not transmitted over HTTP, but if it is known, it should be included nevertheless.
///
/// `http.url` MUST NOT contain credentials passed via URL in form of `https://username:password@www.example.com/`. In such case the attribute&#39;s value should be `https://www.example.com/`.
///
/// # Examples
///
/// - https://www.foo.bar/search?q=OpenTelemetry#SemConv
pub const HTTP_URL: Key = Key::from_static_str("http.url");

/// The full request target as passed in a HTTP request line or equivalent.
///
/// # Examples
///
/// - /path/12314/?q=ddds#123
pub const HTTP_TARGET: Key = Key::from_static_str("http.target");

/// The value of the [HTTP host header](https://tools.ietf.org/html/rfc7230#section-5.4). When the header is empty or not present, this attribute should be the same.
///
/// # Examples
///
/// - www.example.org
pub const HTTP_HOST: Key = Key::from_static_str("http.host");

/// The URI scheme identifying the used protocol.
///
/// # Examples
///
/// - http
/// - https
pub const HTTP_SCHEME: Key = Key::from_static_str("http.scheme");

/// [HTTP response status code](https://tools.ietf.org/html/rfc7231#section-6).
///
/// # Examples
///
/// - 200
pub const HTTP_STATUS_CODE: Key = Key::from_static_str("http.status_code");

/// Kind of HTTP protocol used.
///
/// If `net.transport` is not specified, it can be assumed to be `IP.TCP` except if `http.flavor` is `QUIC`, in which case `IP.UDP` is assumed.
///
/// # Examples
///
/// - 1.0
pub const HTTP_FLAVOR: Key = Key::from_static_str("http.flavor");

/// Value of the [HTTP User-Agent](https://tools.ietf.org/html/rfc7231#section-5.5.3) header sent by the client.
///
/// # Examples
///
/// - CERN-LineMode/2.15 libwww/2.17b3
pub const HTTP_USER_AGENT: Key = Key::from_static_str("http.user_agent");

/// The size of the request payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://tools.ietf.org/html/rfc7230#section-3.3.2) header. For requests using transport encoding, this should be the compressed size.
///
/// # Examples
///
/// - 3495
pub const HTTP_REQUEST_CONTENT_LENGTH: Key = Key::from_static_str("http.request_content_length");

/// The size of the uncompressed request payload body after transport decoding. Not set if transport encoding not used.
///
/// # Examples
///
/// - 5493
pub const HTTP_REQUEST_CONTENT_LENGTH_UNCOMPRESSED: Key =
    Key::from_static_str("http.request_content_length_uncompressed");

/// The size of the response payload body in bytes. This is the number of bytes transferred excluding headers and is often, but not always, present as the [Content-Length](https://tools.ietf.org/html/rfc7230#section-3.3.2) header. For requests using transport encoding, this should be the compressed size.
///
/// # Examples
///
/// - 3495
pub const HTTP_RESPONSE_CONTENT_LENGTH: Key = Key::from_static_str("http.response_content_length");

/// The size of the uncompressed response payload body after transport decoding. Not set if transport encoding not used.
///
/// # Examples
///
/// - 5493
pub const HTTP_RESPONSE_CONTENT_LENGTH_UNCOMPRESSED: Key =
    Key::from_static_str("http.response_content_length_uncompressed");

/// The primary server name of the matched virtual host. This should be obtained via configuration. If no such configuration can be obtained, this attribute MUST NOT be set ( `net.host.name` should be used instead).
///
/// `http.url` is usually not readily available on the server side but would have to be assembled in a cumbersome and sometimes lossy process from other information (see e.g. open-telemetry/opentelemetry-python/pull/148). It is thus preferred to supply the raw data that is available.
///
/// # Examples
///
/// - example.com
pub const HTTP_SERVER_NAME: Key = Key::from_static_str("http.server_name");

/// The matched route (path template).
///
/// # Examples
///
/// - /users/:userID?
pub const HTTP_ROUTE: Key = Key::from_static_str("http.route");

/// The IP address of the original client behind all proxies, if known (e.g. from [X-Forwarded-For](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For)).
///
/// This is not necessarily the same as `net.peer.ip`, which would identify the network-level peer, which may be a proxy.
///
/// # Examples
///
/// - 83.164.160.102
pub const HTTP_CLIENT_IP: Key = Key::from_static_str("http.client_ip");

/// Like `net.peer.ip` but for the host IP. Useful in case of a multi-IP host.
///
/// # Examples
///
/// - 192.168.0.1
pub const NET_HOST_IP: Key = Key::from_static_str("net.host.ip");

/// Like `net.peer.port` but for the host port.
///
/// # Examples
///
/// - 35555
pub const NET_HOST_PORT: Key = Key::from_static_str("net.host.port");

/// Local hostname or similar, see note below.
///
/// # Examples
///
/// - localhost
pub const NET_HOST_NAME: Key = Key::from_static_str("net.host.name");

/// A string identifying the messaging system.
///
/// # Examples
///
/// - kafka
/// - rabbitmq
/// - activemq
pub const MESSAGING_SYSTEM: Key = Key::from_static_str("messaging.system");

/// The message destination name. This might be equal to the span name but is required nevertheless.
///
/// # Examples
///
/// - MyQueue
/// - MyTopic
pub const MESSAGING_DESTINATION: Key = Key::from_static_str("messaging.destination");

/// The kind of message destination.
pub const MESSAGING_DESTINATION_KIND: Key = Key::from_static_str("messaging.destination_kind");

/// A boolean that is true if the message destination is temporary.
pub const MESSAGING_TEMP_DESTINATION: Key = Key::from_static_str("messaging.temp_destination");

/// The name of the transport protocol.
///
/// # Examples
///
/// - AMQP
/// - MQTT
pub const MESSAGING_PROTOCOL: Key = Key::from_static_str("messaging.protocol");

/// The version of the transport protocol.
///
/// # Examples
///
/// - 0.9.1
pub const MESSAGING_PROTOCOL_VERSION: Key = Key::from_static_str("messaging.protocol_version");

/// Connection string.
///
/// # Examples
///
/// - tibjmsnaming://localhost:7222
/// - https://queue.amazonaws.com/80398EXAMPLE/MyQueue
pub const MESSAGING_URL: Key = Key::from_static_str("messaging.url");

/// A value used by the messaging system as an identifier for the message, represented as a string.
///
/// # Examples
///
/// - 452a7c7c7c7048c2f887f61572b18fc2
pub const MESSAGING_MESSAGE_ID: Key = Key::from_static_str("messaging.message_id");

/// The [conversation ID](#conversations) identifying the conversation to which the message belongs, represented as a string. Sometimes called &#34;Correlation ID&#34;.
///
/// # Examples
///
/// - MyConversationId
pub const MESSAGING_CONVERSATION_ID: Key = Key::from_static_str("messaging.conversation_id");

/// The (uncompressed) size of the message payload in bytes. Also use this attribute if it is unknown whether the compressed or uncompressed payload size is reported.
///
/// # Examples
///
/// - 2738
pub const MESSAGING_MESSAGE_PAYLOAD_SIZE_BYTES: Key =
    Key::from_static_str("messaging.message_payload_size_bytes");

/// The compressed size of the message payload in bytes.
///
/// # Examples
///
/// - 2048
pub const MESSAGING_MESSAGE_PAYLOAD_COMPRESSED_SIZE_BYTES: Key =
    Key::from_static_str("messaging.message_payload_compressed_size_bytes");

/// A string containing the function invocation time in the [ISO 8601](https://www.iso.org/iso-8601-date-and-time-format.html) format expressed in [UTC](https://www.w3.org/TR/NOTE-datetime).
///
/// # Examples
///
/// - 2020-01-23T13:47:06Z
pub const FAAS_TIME: Key = Key::from_static_str("faas.time");

/// A string containing the schedule period as [Cron Expression](https://docs.oracle.com/cd/E12058_01/doc/doc.1014/e12030/cron_expressions.htm).
///
/// # Examples
///
/// - 0/5 * * * ? *
pub const FAAS_CRON: Key = Key::from_static_str("faas.cron");

/// A boolean that is true if the serverless function is executed for the first time (aka cold-start).
pub const FAAS_COLDSTART: Key = Key::from_static_str("faas.coldstart");

/// The name of the invoked function.
///
/// SHOULD be equal to the `faas.name` resource attribute of the invoked function.
///
/// # Examples
///
/// - my-function
pub const FAAS_INVOKED_NAME: Key = Key::from_static_str("faas.invoked_name");

/// The cloud provider of the invoked function.
///
/// SHOULD be equal to the `cloud.provider` resource attribute of the invoked function.
///
/// # Examples
///
/// - aws
pub const FAAS_INVOKED_PROVIDER: Key = Key::from_static_str("faas.invoked_provider");

/// The cloud region of the invoked function.
///
/// SHOULD be equal to the `cloud.region` resource attribute of the invoked function.
///
/// # Examples
///
/// - eu-central-1
pub const FAAS_INVOKED_REGION: Key = Key::from_static_str("faas.invoked_region");

/// The [`service.name`](../../resource/semantic_conventions/README.md#service) of the remote service. SHOULD be equal to the actual `service.name` resource attribute of the remote service if any.
///
/// # Examples
///
/// - AuthTokenCache
pub const PEER_SERVICE: Key = Key::from_static_str("peer.service");

/// Username or client_id extracted from the access token or [Authorization](https://tools.ietf.org/html/rfc7235#section-4.2) header in the inbound request from outside the system.
///
/// # Examples
///
/// - username
pub const ENDUSER_ID: Key = Key::from_static_str("enduser.id");

/// Actual/assumed role the client is making the request under extracted from token or application security context.
///
/// # Examples
///
/// - admin
pub const ENDUSER_ROLE: Key = Key::from_static_str("enduser.role");

/// Scopes or granted authorities the client currently possesses extracted from token or application security context. The value would come from the scope associated with an [OAuth 2.0 Access Token](https://tools.ietf.org/html/rfc6749#section-3.3) or an attribute value in a [SAML 2.0 Assertion](http://docs.oasis-open.org/security/saml/Post2.0/sstc-saml-tech-overview-2.0.html).
///
/// # Examples
///
/// - read:message, write:files
pub const ENDUSER_SCOPE: Key = Key::from_static_str("enduser.scope");

/// Current &#34;managed&#34; thread ID (as opposed to OS thread ID).
///
/// # Examples
///
/// - 42
pub const THREAD_ID: Key = Key::from_static_str("thread.id");

/// Current thread name.
///
/// # Examples
///
/// - main
pub const THREAD_NAME: Key = Key::from_static_str("thread.name");

/// The method or function name, or equivalent (usually rightmost part of the code unit&#39;s name).
///
/// # Examples
///
/// - serveRequest
pub const CODE_FUNCTION: Key = Key::from_static_str("code.function");

/// The &#34;namespace&#34; within which `code.function` is defined. Usually the qualified class or module name, such that `code.namespace` + some separator + `code.function` form a unique identifier for the code unit.
///
/// # Examples
///
/// - com.example.MyHttpService
pub const CODE_NAMESPACE: Key = Key::from_static_str("code.namespace");

/// The source code file name that identifies the code unit as uniquely as possible (preferably an absolute file path).
///
/// # Examples
///
/// - /usr/local/MyApplication/content_root/app/index.php
pub const CODE_FILEPATH: Key = Key::from_static_str("code.filepath");

/// The line number in `code.filepath` best representing the operation. It SHOULD point within the code unit named in `code.function`.
///
/// # Examples
///
/// - 42
pub const CODE_LINENO: Key = Key::from_static_str("code.lineno");

/// A string identifying the kind of message consumption as defined in the [Operation names](#operation-names) section above. If the operation is &#34;send&#34;, this attribute MUST NOT be set, since the operation can be inferred from the span kind in that case.
pub const MESSAGING_OPERATION: Key = Key::from_static_str("messaging.operation");

/// Message keys in Kafka are used for grouping alike messages to ensure they&#39;re processed on the same partition. They differ from `messaging.message_id` in that they&#39;re not unique. If the key is `null`, the attribute MUST NOT be set.
///
/// If the key type is not string, it&#39;s string representation has to be supplied for the attribute. If the key has no unambiguous, canonical string form, don&#39;t include its value.
///
/// # Examples
///
/// - myKey
pub const MESSAGING_KAFKA_MESSAGE_KEY: Key = Key::from_static_str("messaging.kafka.message_key");

/// Name of the Kafka Consumer Group that is handling the message. Only applies to consumers, not producers.
///
/// # Examples
///
/// - my-group
pub const MESSAGING_KAFKA_CONSUMER_GROUP: Key =
    Key::from_static_str("messaging.kafka.consumer_group");

/// Client Id for the Consumer or Producer that is handling the message.
///
/// # Examples
///
/// - client-5
pub const MESSAGING_KAFKA_CLIENT_ID: Key = Key::from_static_str("messaging.kafka.client_id");

/// Partition the message is sent to.
///
/// # Examples
///
/// - 2
pub const MESSAGING_KAFKA_PARTITION: Key = Key::from_static_str("messaging.kafka.partition");

/// A boolean that is true if the message is a tombstone.
pub const MESSAGING_KAFKA_TOMBSTONE: Key = Key::from_static_str("messaging.kafka.tombstone");

/// A string identifying the remoting system.
///
/// # Examples
///
/// - grpc
/// - java_rmi
/// - wcf
pub const RPC_SYSTEM: Key = Key::from_static_str("rpc.system");

/// The full name of the service being called, including its package name, if applicable.
///
/// # Examples
///
/// - myservice.EchoService
pub const RPC_SERVICE: Key = Key::from_static_str("rpc.service");

/// The name of the method being called, must be equal to the $method part in the span name.
///
/// # Examples
///
/// - exampleMethod
pub const RPC_METHOD: Key = Key::from_static_str("rpc.method");

/// The [numeric status code](https://github.com/grpc/grpc/blob/v1.33.2/doc/statuscodes.md) of the gRPC request.
///
/// # Examples
///
/// - 0
/// - 1
/// - 16
pub const RPC_GRPC_STATUS_CODE: Key = Key::from_static_str("rpc.grpc.status_code");
