// DO NOT EDIT, this is an auto-generated file
//
// If you want to update the file:
// - Edit the template at scripts/templates/registry/rust/attributes.rs.j2
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
//!     .with_attributes([
//!         KeyValue::new(semconv::trace::CLIENT_ADDRESS, "example.org"),
//!         KeyValue::new(semconv::trace::CLIENT_PORT, 80i64),
//!     ])
//!     .start(&tracer);
//! ```

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_ATTRIBUTE_DEFINITIONS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_ATTRIBUTES_TO_GET;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_CONSISTENT_READ;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_CONSUMED_CAPACITY;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_COUNT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_EXCLUSIVE_START_TABLE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_GLOBAL_SECONDARY_INDEX_UPDATES;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_GLOBAL_SECONDARY_INDEXES;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_INDEX_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_ITEM_COLLECTION_METRICS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_LIMIT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_LOCAL_SECONDARY_INDEXES;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_PROJECTION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_PROVISIONED_READ_CAPACITY;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_PROVISIONED_WRITE_CAPACITY;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_SCAN_FORWARD;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_SCANNED_COUNT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_SEGMENT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_SELECT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_TABLE_COUNT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_TABLE_NAMES;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_DYNAMODB_TOTAL_SEGMENTS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_LAMBDA_INVOKED_ARN;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_REQUEST_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_S3_BUCKET;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_S3_COPY_SOURCE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_S3_DELETE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_S3_KEY;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_S3_PART_NUMBER;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AWS_S3_UPLOAD_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::AZ_SERVICE_REQUEST_ID;

pub use crate::attribute::CLIENT_ADDRESS;

pub use crate::attribute::CLIENT_PORT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CLOUD_RESOURCE_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CLOUDEVENTS_EVENT_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CLOUDEVENTS_EVENT_SOURCE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CLOUDEVENTS_EVENT_SPEC_VERSION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CLOUDEVENTS_EVENT_SUBJECT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CLOUDEVENTS_EVENT_TYPE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CODE_COLUMN;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CODE_FILEPATH;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CODE_FUNCTION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CODE_LINENO;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CODE_NAMESPACE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::CODE_STACKTRACE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_CASSANDRA_CONSISTENCY_LEVEL;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_CASSANDRA_COORDINATOR_DC;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_CASSANDRA_COORDINATOR_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_CASSANDRA_IDEMPOTENCE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_CASSANDRA_PAGE_SIZE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_CASSANDRA_SPECULATIVE_EXECUTION_COUNT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_COLLECTION_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_COSMOSDB_CLIENT_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_COSMOSDB_CONNECTION_MODE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_COSMOSDB_OPERATION_TYPE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_COSMOSDB_REQUEST_CHARGE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_COSMOSDB_REQUEST_CONTENT_LENGTH;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_COSMOSDB_STATUS_CODE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_COSMOSDB_SUB_STATUS_CODE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_ELASTICSEARCH_NODE_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_ELASTICSEARCH_PATH_PARTS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_NAMESPACE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_OPERATION_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_QUERY_PARAMETER;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_QUERY_TEXT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::DB_SYSTEM;

#[cfg(feature = "semconv_experimental")]
#[allow(deprecated)]
pub use crate::attribute::ENDUSER_ID;

#[cfg(feature = "semconv_experimental")]
#[allow(deprecated)]
pub use crate::attribute::ENDUSER_ROLE;

#[cfg(feature = "semconv_experimental")]
#[allow(deprecated)]
pub use crate::attribute::ENDUSER_SCOPE;

pub use crate::attribute::ERROR_TYPE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::EVENT_NAME;

pub use crate::attribute::EXCEPTION_ESCAPED;

pub use crate::attribute::EXCEPTION_MESSAGE;

pub use crate::attribute::EXCEPTION_STACKTRACE;

pub use crate::attribute::EXCEPTION_TYPE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_COLDSTART;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_CRON;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_DOCUMENT_COLLECTION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_DOCUMENT_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_DOCUMENT_OPERATION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_DOCUMENT_TIME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_INVOCATION_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_INVOKED_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_INVOKED_PROVIDER;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_INVOKED_REGION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_TIME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FAAS_TRIGGER;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FEATURE_FLAG_KEY;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FEATURE_FLAG_PROVIDER_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::FEATURE_FLAG_VARIANT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_COMPLETION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_OPERATION_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_PROMPT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_REQUEST_FREQUENCY_PENALTY;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_REQUEST_MAX_TOKENS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_REQUEST_MODEL;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_REQUEST_PRESENCE_PENALTY;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_REQUEST_STOP_SEQUENCES;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_REQUEST_TEMPERATURE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_REQUEST_TOP_K;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_REQUEST_TOP_P;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_RESPONSE_FINISH_REASONS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_RESPONSE_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_RESPONSE_MODEL;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_SYSTEM;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_USAGE_INPUT_TOKENS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GEN_AI_USAGE_OUTPUT_TOKENS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GRAPHQL_DOCUMENT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GRAPHQL_OPERATION_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::GRAPHQL_OPERATION_TYPE;

pub use crate::attribute::HTTP_REQUEST_HEADER;

pub use crate::attribute::HTTP_REQUEST_METHOD;

pub use crate::attribute::HTTP_REQUEST_METHOD_ORIGINAL;

pub use crate::attribute::HTTP_REQUEST_RESEND_COUNT;

pub use crate::attribute::HTTP_RESPONSE_HEADER;

pub use crate::attribute::HTTP_RESPONSE_STATUS_CODE;

pub use crate::attribute::HTTP_ROUTE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_BATCH_MESSAGE_COUNT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_CLIENT_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_CONSUMER_GROUP_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_DESTINATION_ANONYMOUS;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_DESTINATION_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_DESTINATION_PARTITION_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_DESTINATION_SUBSCRIPTION_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_DESTINATION_TEMPLATE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_DESTINATION_TEMPORARY;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_MESSAGE_BODY_SIZE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_MESSAGE_CONVERSATION_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_MESSAGE_ENVELOPE_SIZE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_MESSAGE_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_OPERATION_NAME;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_OPERATION_TYPE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::MESSAGING_SYSTEM;

pub use crate::attribute::NETWORK_LOCAL_ADDRESS;

pub use crate::attribute::NETWORK_LOCAL_PORT;

pub use crate::attribute::NETWORK_PEER_ADDRESS;

pub use crate::attribute::NETWORK_PEER_PORT;

pub use crate::attribute::NETWORK_PROTOCOL_NAME;

pub use crate::attribute::NETWORK_PROTOCOL_VERSION;

pub use crate::attribute::NETWORK_TRANSPORT;

pub use crate::attribute::NETWORK_TYPE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::OPENTRACING_REF_TYPE;

pub use crate::attribute::OTEL_STATUS_CODE;

pub use crate::attribute::OTEL_STATUS_DESCRIPTION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::PEER_SERVICE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_CONNECT_RPC_ERROR_CODE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_CONNECT_RPC_REQUEST_METADATA;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_CONNECT_RPC_RESPONSE_METADATA;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_GRPC_REQUEST_METADATA;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_GRPC_RESPONSE_METADATA;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_GRPC_STATUS_CODE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_JSONRPC_ERROR_CODE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_JSONRPC_ERROR_MESSAGE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_JSONRPC_REQUEST_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_JSONRPC_VERSION;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_MESSAGE_COMPRESSED_SIZE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_MESSAGE_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_MESSAGE_TYPE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_MESSAGE_UNCOMPRESSED_SIZE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_METHOD;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_SERVICE;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::RPC_SYSTEM;

pub use crate::attribute::SERVER_ADDRESS;

pub use crate::attribute::SERVER_PORT;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::THREAD_ID;

#[cfg(feature = "semconv_experimental")]
pub use crate::attribute::THREAD_NAME;

pub use crate::attribute::URL_FULL;

pub use crate::attribute::URL_PATH;

pub use crate::attribute::URL_QUERY;

pub use crate::attribute::URL_SCHEME;

pub use crate::attribute::USER_AGENT_ORIGINAL;
