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
//! ```rust
//! use opentelemetry::KeyValue;
//! use opentelemetry::{global, trace::Tracer as _};
//! use opentelemetry_semantic_conventions as semconv;
//!
//! let tracer = global::tracer("my-component");
//! let _span = tracer
//!     .span_builder("span-name")
//!     .with_attributes(vec![
//!         KeyValue::new(semconv::trace::CLIENT_ADDRESS, "example.org"),
//!         KeyValue::new(semconv::trace::CLIENT_PORT, 80i64),
//!     ])
//!     .start(&tracer);
//! ```
pub use crate::attribute::AWS_DYNAMODB_ATTRIBUTES_TO_GET;
pub use crate::attribute::AWS_DYNAMODB_ATTRIBUTE_DEFINITIONS;
pub use crate::attribute::AWS_DYNAMODB_CONSISTENT_READ;
pub use crate::attribute::AWS_DYNAMODB_CONSUMED_CAPACITY;
pub use crate::attribute::AWS_DYNAMODB_COUNT;
pub use crate::attribute::AWS_DYNAMODB_EXCLUSIVE_START_TABLE;
pub use crate::attribute::AWS_DYNAMODB_GLOBAL_SECONDARY_INDEXES;
pub use crate::attribute::AWS_DYNAMODB_GLOBAL_SECONDARY_INDEX_UPDATES;
pub use crate::attribute::AWS_DYNAMODB_INDEX_NAME;
pub use crate::attribute::AWS_DYNAMODB_ITEM_COLLECTION_METRICS;
pub use crate::attribute::AWS_DYNAMODB_LIMIT;
pub use crate::attribute::AWS_DYNAMODB_LOCAL_SECONDARY_INDEXES;
pub use crate::attribute::AWS_DYNAMODB_PROJECTION;
pub use crate::attribute::AWS_DYNAMODB_PROVISIONED_READ_CAPACITY;
pub use crate::attribute::AWS_DYNAMODB_PROVISIONED_WRITE_CAPACITY;
pub use crate::attribute::AWS_DYNAMODB_SCANNED_COUNT;
pub use crate::attribute::AWS_DYNAMODB_SCAN_FORWARD;
pub use crate::attribute::AWS_DYNAMODB_SEGMENT;
pub use crate::attribute::AWS_DYNAMODB_SELECT;
pub use crate::attribute::AWS_DYNAMODB_TABLE_COUNT;
pub use crate::attribute::AWS_DYNAMODB_TABLE_NAMES;
pub use crate::attribute::AWS_DYNAMODB_TOTAL_SEGMENTS;
pub use crate::attribute::AWS_LAMBDA_INVOKED_ARN;
pub use crate::attribute::AWS_REQUEST_ID;
pub use crate::attribute::AWS_S3_BUCKET;
pub use crate::attribute::AWS_S3_COPY_SOURCE;
pub use crate::attribute::AWS_S3_DELETE;
pub use crate::attribute::AWS_S3_KEY;
pub use crate::attribute::AWS_S3_PART_NUMBER;
pub use crate::attribute::AWS_S3_UPLOAD_ID;
pub use crate::attribute::CLIENT_ADDRESS;
pub use crate::attribute::CLIENT_PORT;
pub use crate::attribute::CLOUDEVENTS_EVENT_ID;
pub use crate::attribute::CLOUDEVENTS_EVENT_SOURCE;
pub use crate::attribute::CLOUDEVENTS_EVENT_SPEC_VERSION;
pub use crate::attribute::CLOUDEVENTS_EVENT_SUBJECT;
pub use crate::attribute::CLOUDEVENTS_EVENT_TYPE;
pub use crate::attribute::CLOUD_RESOURCE_ID;
pub use crate::attribute::CODE_COLUMN;
pub use crate::attribute::CODE_FILEPATH;
pub use crate::attribute::CODE_FUNCTION;
pub use crate::attribute::CODE_LINENO;
pub use crate::attribute::CODE_NAMESPACE;
pub use crate::attribute::CODE_STACKTRACE;
pub use crate::attribute::DB_CASSANDRA_CONSISTENCY_LEVEL;
pub use crate::attribute::DB_CASSANDRA_COORDINATOR_DC;
pub use crate::attribute::DB_CASSANDRA_COORDINATOR_ID;
pub use crate::attribute::DB_CASSANDRA_IDEMPOTENCE;
pub use crate::attribute::DB_CASSANDRA_PAGE_SIZE;
pub use crate::attribute::DB_CASSANDRA_SPECULATIVE_EXECUTION_COUNT;
pub use crate::attribute::DB_COLLECTION_NAME;
pub use crate::attribute::DB_COSMOSDB_CLIENT_ID;
pub use crate::attribute::DB_COSMOSDB_CONNECTION_MODE;
pub use crate::attribute::DB_COSMOSDB_OPERATION_TYPE;
pub use crate::attribute::DB_COSMOSDB_REQUEST_CHARGE;
pub use crate::attribute::DB_COSMOSDB_REQUEST_CONTENT_LENGTH;
pub use crate::attribute::DB_COSMOSDB_STATUS_CODE;
pub use crate::attribute::DB_COSMOSDB_SUB_STATUS_CODE;
pub use crate::attribute::DB_ELASTICSEARCH_CLUSTER_NAME;
pub use crate::attribute::DB_ELASTICSEARCH_NODE_NAME;
pub use crate::attribute::DB_NAMESPACE;
pub use crate::attribute::DB_OPERATION_NAME;
pub use crate::attribute::DB_QUERY_TEXT;
pub use crate::attribute::DB_SYSTEM;
pub use crate::attribute::ENDUSER_ID;
pub use crate::attribute::ENDUSER_ROLE;
pub use crate::attribute::ENDUSER_SCOPE;
pub use crate::attribute::ERROR_TYPE;
pub use crate::attribute::EXCEPTION_ESCAPED;
pub use crate::attribute::EXCEPTION_MESSAGE;
pub use crate::attribute::EXCEPTION_STACKTRACE;
pub use crate::attribute::EXCEPTION_TYPE;
pub use crate::attribute::FAAS_COLDSTART;
pub use crate::attribute::FAAS_CRON;
pub use crate::attribute::FAAS_DOCUMENT_COLLECTION;
pub use crate::attribute::FAAS_DOCUMENT_NAME;
pub use crate::attribute::FAAS_DOCUMENT_OPERATION;
pub use crate::attribute::FAAS_DOCUMENT_TIME;
pub use crate::attribute::FAAS_INVOCATION_ID;
pub use crate::attribute::FAAS_INVOKED_NAME;
pub use crate::attribute::FAAS_INVOKED_PROVIDER;
pub use crate::attribute::FAAS_INVOKED_REGION;
pub use crate::attribute::FAAS_TIME;
pub use crate::attribute::FAAS_TRIGGER;
pub use crate::attribute::FEATURE_FLAG_KEY;
pub use crate::attribute::FEATURE_FLAG_PROVIDER_NAME;
pub use crate::attribute::FEATURE_FLAG_VARIANT;
pub use crate::attribute::GEN_AI_COMPLETION;
pub use crate::attribute::GEN_AI_PROMPT;
pub use crate::attribute::GEN_AI_REQUEST_MAX_TOKENS;
pub use crate::attribute::GEN_AI_REQUEST_MODEL;
pub use crate::attribute::GEN_AI_REQUEST_TEMPERATURE;
pub use crate::attribute::GEN_AI_REQUEST_TOP_P;
pub use crate::attribute::GEN_AI_RESPONSE_FINISH_REASONS;
pub use crate::attribute::GEN_AI_RESPONSE_ID;
pub use crate::attribute::GEN_AI_RESPONSE_MODEL;
pub use crate::attribute::GEN_AI_SYSTEM;
pub use crate::attribute::GEN_AI_USAGE_COMPLETION_TOKENS;
pub use crate::attribute::GEN_AI_USAGE_PROMPT_TOKENS;
pub use crate::attribute::GRAPHQL_DOCUMENT;
pub use crate::attribute::GRAPHQL_OPERATION_NAME;
pub use crate::attribute::GRAPHQL_OPERATION_TYPE;
pub use crate::attribute::HTTP_REQUEST_METHOD;
pub use crate::attribute::HTTP_REQUEST_METHOD_ORIGINAL;
pub use crate::attribute::HTTP_REQUEST_RESEND_COUNT;
pub use crate::attribute::HTTP_RESPONSE_STATUS_CODE;
pub use crate::attribute::HTTP_ROUTE;
pub use crate::attribute::MESSAGING_BATCH_MESSAGE_COUNT;
pub use crate::attribute::MESSAGING_CLIENT_ID;
pub use crate::attribute::MESSAGING_DESTINATION_ANONYMOUS;
pub use crate::attribute::MESSAGING_DESTINATION_NAME;
pub use crate::attribute::MESSAGING_DESTINATION_PARTITION_ID;
pub use crate::attribute::MESSAGING_DESTINATION_TEMPLATE;
pub use crate::attribute::MESSAGING_DESTINATION_TEMPORARY;
pub use crate::attribute::MESSAGING_MESSAGE_BODY_SIZE;
pub use crate::attribute::MESSAGING_MESSAGE_CONVERSATION_ID;
pub use crate::attribute::MESSAGING_MESSAGE_ENVELOPE_SIZE;
pub use crate::attribute::MESSAGING_MESSAGE_ID;
pub use crate::attribute::MESSAGING_OPERATION_NAME;
pub use crate::attribute::MESSAGING_OPERATION_TYPE;
pub use crate::attribute::MESSAGING_SYSTEM;
pub use crate::attribute::NETWORK_LOCAL_ADDRESS;
pub use crate::attribute::NETWORK_LOCAL_PORT;
pub use crate::attribute::NETWORK_PEER_ADDRESS;
pub use crate::attribute::NETWORK_PEER_PORT;
pub use crate::attribute::NETWORK_PROTOCOL_NAME;
pub use crate::attribute::NETWORK_PROTOCOL_VERSION;
pub use crate::attribute::NETWORK_TRANSPORT;
pub use crate::attribute::NETWORK_TYPE;
pub use crate::attribute::OPENTRACING_REF_TYPE;
pub use crate::attribute::OTEL_STATUS_CODE;
pub use crate::attribute::OTEL_STATUS_DESCRIPTION;
pub use crate::attribute::PEER_SERVICE;
pub use crate::attribute::RPC_CONNECT_RPC_ERROR_CODE;
pub use crate::attribute::RPC_GRPC_STATUS_CODE;
pub use crate::attribute::RPC_JSONRPC_ERROR_CODE;
pub use crate::attribute::RPC_JSONRPC_ERROR_MESSAGE;
pub use crate::attribute::RPC_JSONRPC_REQUEST_ID;
pub use crate::attribute::RPC_JSONRPC_VERSION;
pub use crate::attribute::RPC_MESSAGE_COMPRESSED_SIZE;
pub use crate::attribute::RPC_MESSAGE_ID;
pub use crate::attribute::RPC_MESSAGE_TYPE;
pub use crate::attribute::RPC_MESSAGE_UNCOMPRESSED_SIZE;
pub use crate::attribute::RPC_METHOD;
pub use crate::attribute::RPC_SERVICE;
pub use crate::attribute::RPC_SYSTEM;
pub use crate::attribute::SERVER_ADDRESS;
pub use crate::attribute::SERVER_PORT;
pub use crate::attribute::THREAD_ID;
pub use crate::attribute::THREAD_NAME;
pub use crate::attribute::URL_FULL;
pub use crate::attribute::URL_PATH;
pub use crate::attribute::URL_QUERY;
pub use crate::attribute::URL_SCHEME;
pub use crate::attribute::USER_AGENT_ORIGINAL;
