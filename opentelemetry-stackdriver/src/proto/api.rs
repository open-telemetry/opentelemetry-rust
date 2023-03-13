/// Defines the HTTP configuration for an API service. It contains a list of
/// \[HttpRule][google.api.HttpRule\], each specifying the mapping of an RPC method
/// to one or more HTTP REST API methods.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Http {
    /// A list of HTTP configuration rules that apply to individual API methods.
    ///
    /// **NOTE:** All service configuration rules follow "last one wins" order.
    #[prost(message, repeated, tag = "1")]
    pub rules: ::prost::alloc::vec::Vec<HttpRule>,
    /// When set to true, URL path parameters will be fully URI-decoded except in
    /// cases of single segment matches in reserved expansion, where "%2F" will be
    /// left encoded.
    ///
    /// The default behavior is to not decode RFC 6570 reserved characters in multi
    /// segment matches.
    #[prost(bool, tag = "2")]
    pub fully_decode_reserved_expansion: bool,
}
/// # gRPC Transcoding
///
/// gRPC Transcoding is a feature for mapping between a gRPC method and one or
/// more HTTP REST endpoints. It allows developers to build a single API service
/// that supports both gRPC APIs and REST APIs. Many systems, including [Google
/// APIs](<https://github.com/googleapis/googleapis>),
/// [Cloud Endpoints](<https://cloud.google.com/endpoints>), [gRPC
/// Gateway](<https://github.com/grpc-ecosystem/grpc-gateway>),
/// and \[Envoy\](<https://github.com/envoyproxy/envoy>) proxy support this feature
/// and use it for large scale production services.
///
/// `HttpRule` defines the schema of the gRPC/REST mapping. The mapping specifies
/// how different portions of the gRPC request message are mapped to the URL
/// path, URL query parameters, and HTTP request body. It also controls how the
/// gRPC response message is mapped to the HTTP response body. `HttpRule` is
/// typically specified as an `google.api.http` annotation on the gRPC method.
///
/// Each mapping specifies a URL path template and an HTTP method. The path
/// template may refer to one or more fields in the gRPC request message, as long
/// as each field is a non-repeated field with a primitive (non-message) type.
/// The path template controls how fields of the request message are mapped to
/// the URL path.
///
/// Example:
///
///      service Messaging {
///        rpc GetMessage(GetMessageRequest) returns (Message) {
///          option (google.api.http) = {
///              get: "/v1/{name=messages/*}"
///          };
///        }
///      }
///      message GetMessageRequest {
///        string name = 1; // Mapped to URL path.
///      }
///      message Message {
///        string text = 1; // The resource content.
///      }
///
/// This enables an HTTP REST to gRPC mapping as below:
///
/// HTTP | gRPC
/// -----|-----
/// `GET /v1/messages/123456`  | `GetMessage(name: "messages/123456")`
///
/// Any fields in the request message which are not bound by the path template
/// automatically become HTTP query parameters if there is no HTTP request body.
/// For example:
///
///      service Messaging {
///        rpc GetMessage(GetMessageRequest) returns (Message) {
///          option (google.api.http) = {
///              get:"/v1/messages/{message_id}"
///          };
///        }
///      }
///      message GetMessageRequest {
///        message SubMessage {
///          string subfield = 1;
///        }
///        string message_id = 1; // Mapped to URL path.
///        int64 revision = 2;    // Mapped to URL query parameter `revision`.
///        SubMessage sub = 3;    // Mapped to URL query parameter `sub.subfield`.
///      }
///
/// This enables a HTTP JSON to RPC mapping as below:
///
/// HTTP | gRPC
/// -----|-----
/// `GET /v1/messages/123456?revision=2&sub.subfield=foo` |
/// `GetMessage(message_id: "123456" revision: 2 sub: SubMessage(subfield:
/// "foo"))`
///
/// Note that fields which are mapped to URL query parameters must have a
/// primitive type or a repeated primitive type or a non-repeated message type.
/// In the case of a repeated type, the parameter can be repeated in the URL
/// as `...?param=A&param=B`. In the case of a message type, each field of the
/// message is mapped to a separate parameter, such as
/// `...?foo.a=A&foo.b=B&foo.c=C`.
///
/// For HTTP methods that allow a request body, the `body` field
/// specifies the mapping. Consider a REST update method on the
/// message resource collection:
///
///      service Messaging {
///        rpc UpdateMessage(UpdateMessageRequest) returns (Message) {
///          option (google.api.http) = {
///            patch: "/v1/messages/{message_id}"
///            body: "message"
///          };
///        }
///      }
///      message UpdateMessageRequest {
///        string message_id = 1; // mapped to the URL
///        Message message = 2;   // mapped to the body
///      }
///
/// The following HTTP JSON to RPC mapping is enabled, where the
/// representation of the JSON in the request body is determined by
/// protos JSON encoding:
///
/// HTTP | gRPC
/// -----|-----
/// `PATCH /v1/messages/123456 { "text": "Hi!" }` | `UpdateMessage(message_id:
/// "123456" message { text: "Hi!" })`
///
/// The special name `*` can be used in the body mapping to define that
/// every field not bound by the path template should be mapped to the
/// request body.  This enables the following alternative definition of
/// the update method:
///
///      service Messaging {
///        rpc UpdateMessage(Message) returns (Message) {
///          option (google.api.http) = {
///            patch: "/v1/messages/{message_id}"
///            body: "*"
///          };
///        }
///      }
///      message Message {
///        string message_id = 1;
///        string text = 2;
///      }
///
///
/// The following HTTP JSON to RPC mapping is enabled:
///
/// HTTP | gRPC
/// -----|-----
/// `PATCH /v1/messages/123456 { "text": "Hi!" }` | `UpdateMessage(message_id:
/// "123456" text: "Hi!")`
///
/// Note that when using `*` in the body mapping, it is not possible to
/// have HTTP parameters, as all fields not bound by the path end in
/// the body. This makes this option more rarely used in practice when
/// defining REST APIs. The common usage of `*` is in custom methods
/// which don't use the URL at all for transferring data.
///
/// It is possible to define multiple HTTP methods for one RPC by using
/// the `additional_bindings` option. Example:
///
///      service Messaging {
///        rpc GetMessage(GetMessageRequest) returns (Message) {
///          option (google.api.http) = {
///            get: "/v1/messages/{message_id}"
///            additional_bindings {
///              get: "/v1/users/{user_id}/messages/{message_id}"
///            }
///          };
///        }
///      }
///      message GetMessageRequest {
///        string message_id = 1;
///        string user_id = 2;
///      }
///
/// This enables the following two alternative HTTP JSON to RPC mappings:
///
/// HTTP | gRPC
/// -----|-----
/// `GET /v1/messages/123456` | `GetMessage(message_id: "123456")`
/// `GET /v1/users/me/messages/123456` | `GetMessage(user_id: "me" message_id:
/// "123456")`
///
/// ## Rules for HTTP mapping
///
/// 1. Leaf request fields (recursive expansion nested messages in the request
///     message) are classified into three categories:
///     - Fields referred by the path template. They are passed via the URL path.
///     - Fields referred by the \[HttpRule.body][google.api.HttpRule.body\]. They are passed via the HTTP
///       request body.
///     - All other fields are passed via the URL query parameters, and the
///       parameter name is the field path in the request message. A repeated
///       field can be represented as multiple query parameters under the same
///       name.
///   2. If \[HttpRule.body][google.api.HttpRule.body\] is "*", there is no URL query parameter, all fields
///      are passed via URL path and HTTP request body.
///   3. If \[HttpRule.body][google.api.HttpRule.body\] is omitted, there is no HTTP request body, all
///      fields are passed via URL path and URL query parameters.
///
/// ### Path template syntax
///
///      Template = "/" Segments [ Verb ] ;
///      Segments = Segment { "/" Segment } ;
///      Segment  = "*" | "**" | LITERAL | Variable ;
///      Variable = "{" FieldPath [ "=" Segments ] "}" ;
///      FieldPath = IDENT { "." IDENT } ;
///      Verb     = ":" LITERAL ;
///
/// The syntax `*` matches a single URL path segment. The syntax `**` matches
/// zero or more URL path segments, which must be the last part of the URL path
/// except the `Verb`.
///
/// The syntax `Variable` matches part of the URL path as specified by its
/// template. A variable template must not contain other variables. If a variable
/// matches a single path segment, its template may be omitted, e.g. `{var}`
/// is equivalent to `{var=*}`.
///
/// The syntax `LITERAL` matches literal text in the URL path. If the `LITERAL`
/// contains any reserved character, such characters should be percent-encoded
/// before the matching.
///
/// If a variable contains exactly one path segment, such as `"{var}"` or
/// `"{var=*}"`, when such a variable is expanded into a URL path on the client
/// side, all characters except `\[-_.~0-9a-zA-Z\]` are percent-encoded. The
/// server side does the reverse decoding. Such variables show up in the
/// [Discovery
/// Document](<https://developers.google.com/discovery/v1/reference/apis>) as
/// `{var}`.
///
/// If a variable contains multiple path segments, such as `"{var=foo/*}"`
/// or `"{var=**}"`, when such a variable is expanded into a URL path on the
/// client side, all characters except `\[-_.~/0-9a-zA-Z\]` are percent-encoded.
/// The server side does the reverse decoding, except "%2F" and "%2f" are left
/// unchanged. Such variables show up in the
/// [Discovery
/// Document](<https://developers.google.com/discovery/v1/reference/apis>) as
/// `{+var}`.
///
/// ## Using gRPC API Service Configuration
///
/// gRPC API Service Configuration (service config) is a configuration language
/// for configuring a gRPC service to become a user-facing product. The
/// service config is simply the YAML representation of the `google.api.Service`
/// proto message.
///
/// As an alternative to annotating your proto file, you can configure gRPC
/// transcoding in your service config YAML files. You do this by specifying a
/// `HttpRule` that maps the gRPC method to a REST endpoint, achieving the same
/// effect as the proto annotation. This can be particularly useful if you
/// have a proto that is reused in multiple services. Note that any transcoding
/// specified in the service config will override any matching transcoding
/// configuration in the proto.
///
/// Example:
///
///      http:
///        rules:
///          # Selects a gRPC method and applies HttpRule to it.
///          - selector: example.v1.Messaging.GetMessage
///            get: /v1/messages/{message_id}/{sub.subfield}
///
/// ## Special notes
///
/// When gRPC Transcoding is used to map a gRPC to JSON REST endpoints, the
/// proto to JSON conversion must follow the [proto3
/// specification](<https://developers.google.com/protocol-buffers/docs/proto3#json>).
///
/// While the single segment variable follows the semantics of
/// [RFC 6570](<https://tools.ietf.org/html/rfc6570>) Section 3.2.2 Simple String
/// Expansion, the multi segment variable **does not** follow RFC 6570 Section
/// 3.2.3 Reserved Expansion. The reason is that the Reserved Expansion
/// does not expand special characters like `?` and `#`, which would lead
/// to invalid URLs. As the result, gRPC Transcoding uses a custom encoding
/// for multi segment variables.
///
/// The path variables **must not** refer to any repeated or mapped field,
/// because client libraries are not capable of handling such variable expansion.
///
/// The path variables **must not** capture the leading "/" character. The reason
/// is that the most common use case "{var}" does not capture the leading "/"
/// character. For consistency, all path variables must share the same behavior.
///
/// Repeated message fields must not be mapped to URL query parameters, because
/// no client library can support such complicated mapping.
///
/// If an API needs to use a JSON array for request or response body, it can map
/// the request or response body to a repeated field. However, some gRPC
/// Transcoding implementations may not support this feature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpRule {
    /// Selects a method to which this rule applies.
    ///
    /// Refer to \[selector][google.api.DocumentationRule.selector\] for syntax details.
    #[prost(string, tag = "1")]
    pub selector: ::prost::alloc::string::String,
    /// The name of the request field whose value is mapped to the HTTP request
    /// body, or `*` for mapping all request fields not captured by the path
    /// pattern to the HTTP body, or omitted for not having any HTTP request body.
    ///
    /// NOTE: the referred field must be present at the top-level of the request
    /// message type.
    #[prost(string, tag = "7")]
    pub body: ::prost::alloc::string::String,
    /// Optional. The name of the response field whose value is mapped to the HTTP
    /// response body. When omitted, the entire response message will be used
    /// as the HTTP response body.
    ///
    /// NOTE: The referred field must be present at the top-level of the response
    /// message type.
    #[prost(string, tag = "12")]
    pub response_body: ::prost::alloc::string::String,
    /// Additional HTTP bindings for the selector. Nested bindings must
    /// not contain an `additional_bindings` field themselves (that is,
    /// the nesting may only be one level deep).
    #[prost(message, repeated, tag = "11")]
    pub additional_bindings: ::prost::alloc::vec::Vec<HttpRule>,
    /// Determines the URL pattern is matched by this rules. This pattern can be
    /// used with any of the {get|put|post|delete|patch} methods. A custom method
    /// can be defined using the 'custom' field.
    #[prost(oneof = "http_rule::Pattern", tags = "2, 3, 4, 5, 6, 8")]
    pub pattern: ::core::option::Option<http_rule::Pattern>,
}
/// Nested message and enum types in `HttpRule`.
pub mod http_rule {
    /// Determines the URL pattern is matched by this rules. This pattern can be
    /// used with any of the {get|put|post|delete|patch} methods. A custom method
    /// can be defined using the 'custom' field.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Pattern {
        /// Maps to HTTP GET. Used for listing and getting information about
        /// resources.
        #[prost(string, tag = "2")]
        Get(::prost::alloc::string::String),
        /// Maps to HTTP PUT. Used for replacing a resource.
        #[prost(string, tag = "3")]
        Put(::prost::alloc::string::String),
        /// Maps to HTTP POST. Used for creating a resource or performing an action.
        #[prost(string, tag = "4")]
        Post(::prost::alloc::string::String),
        /// Maps to HTTP DELETE. Used for deleting a resource.
        #[prost(string, tag = "5")]
        Delete(::prost::alloc::string::String),
        /// Maps to HTTP PATCH. Used for updating a resource.
        #[prost(string, tag = "6")]
        Patch(::prost::alloc::string::String),
        /// The custom pattern is used for specifying an HTTP method that is not
        /// included in the `pattern` field, such as HEAD, or "*" to leave the
        /// HTTP method unspecified for this rule. The wild-card rule is useful
        /// for services that provide content to Web (HTML) clients.
        #[prost(message, tag = "8")]
        Custom(super::CustomHttpPattern),
    }
}
/// A custom pattern is used for defining custom HTTP verb.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CustomHttpPattern {
    /// The name of this custom HTTP verb.
    #[prost(string, tag = "1")]
    pub kind: ::prost::alloc::string::String,
    /// The path matched by this custom verb.
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
}
/// The launch stage as defined by [Google Cloud Platform
/// Launch Stages](<https://cloud.google.com/terms/launch-stages>).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum LaunchStage {
    /// Do not use this default value.
    Unspecified = 0,
    /// The feature is not yet implemented. Users can not use it.
    Unimplemented = 6,
    /// Prelaunch features are hidden from users and are only visible internally.
    Prelaunch = 7,
    /// Early Access features are limited to a closed group of testers. To use
    /// these features, you must sign up in advance and sign a Trusted Tester
    /// agreement (which includes confidentiality provisions). These features may
    /// be unstable, changed in backward-incompatible ways, and are not
    /// guaranteed to be released.
    EarlyAccess = 1,
    /// Alpha is a limited availability test for releases before they are cleared
    /// for widespread use. By Alpha, all significant design issues are resolved
    /// and we are in the process of verifying functionality. Alpha customers
    /// need to apply for access, agree to applicable terms, and have their
    /// projects allowlisted. Alpha releases don't have to be feature complete,
    /// no SLAs are provided, and there are no technical support obligations, but
    /// they will be far enough along that customers can actually use them in
    /// test environments or for limited-use tests -- just like they would in
    /// normal production cases.
    Alpha = 2,
    /// Beta is the point at which we are ready to open a release for any
    /// customer to use. There are no SLA or technical support obligations in a
    /// Beta release. Products will be complete from a feature perspective, but
    /// may have some open outstanding issues. Beta releases are suitable for
    /// limited production use cases.
    Beta = 3,
    /// GA features are open to all developers and are considered stable and
    /// fully qualified for production use.
    Ga = 4,
    /// Deprecated features are scheduled to be shut down and removed. For more
    /// information, see the "Deprecation Policy" section of our [Terms of
    /// Service](<https://cloud.google.com/terms/>)
    /// and the [Google Cloud Platform Subject to the Deprecation
    /// Policy](<https://cloud.google.com/terms/deprecation>) documentation.
    Deprecated = 5,
}
impl LaunchStage {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            LaunchStage::Unspecified => "LAUNCH_STAGE_UNSPECIFIED",
            LaunchStage::Unimplemented => "UNIMPLEMENTED",
            LaunchStage::Prelaunch => "PRELAUNCH",
            LaunchStage::EarlyAccess => "EARLY_ACCESS",
            LaunchStage::Alpha => "ALPHA",
            LaunchStage::Beta => "BETA",
            LaunchStage::Ga => "GA",
            LaunchStage::Deprecated => "DEPRECATED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "LAUNCH_STAGE_UNSPECIFIED" => Some(Self::Unspecified),
            "UNIMPLEMENTED" => Some(Self::Unimplemented),
            "PRELAUNCH" => Some(Self::Prelaunch),
            "EARLY_ACCESS" => Some(Self::EarlyAccess),
            "ALPHA" => Some(Self::Alpha),
            "BETA" => Some(Self::Beta),
            "GA" => Some(Self::Ga),
            "DEPRECATED" => Some(Self::Deprecated),
            _ => None,
        }
    }
}
/// Required information for every language.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommonLanguageSettings {
    /// Link to automatically generated reference documentation.  Example:
    /// <https://cloud.google.com/nodejs/docs/reference/asset/latest>
    #[deprecated]
    #[prost(string, tag = "1")]
    pub reference_docs_uri: ::prost::alloc::string::String,
    /// The destination where API teams want this client library to be published.
    #[prost(enumeration = "ClientLibraryDestination", repeated, tag = "2")]
    pub destinations: ::prost::alloc::vec::Vec<i32>,
}
/// Details about how and where to publish client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientLibrarySettings {
    /// Version of the API to apply these settings to.
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    /// Launch stage of this version of the API.
    #[prost(enumeration = "LaunchStage", tag = "2")]
    pub launch_stage: i32,
    /// When using transport=rest, the client request will encode enums as
    /// numbers rather than strings.
    #[prost(bool, tag = "3")]
    pub rest_numeric_enums: bool,
    /// Settings for legacy Java features, supported in the Service YAML.
    #[prost(message, optional, tag = "21")]
    pub java_settings: ::core::option::Option<JavaSettings>,
    /// Settings for C++ client libraries.
    #[prost(message, optional, tag = "22")]
    pub cpp_settings: ::core::option::Option<CppSettings>,
    /// Settings for PHP client libraries.
    #[prost(message, optional, tag = "23")]
    pub php_settings: ::core::option::Option<PhpSettings>,
    /// Settings for Python client libraries.
    #[prost(message, optional, tag = "24")]
    pub python_settings: ::core::option::Option<PythonSettings>,
    /// Settings for Node client libraries.
    #[prost(message, optional, tag = "25")]
    pub node_settings: ::core::option::Option<NodeSettings>,
    /// Settings for .NET client libraries.
    #[prost(message, optional, tag = "26")]
    pub dotnet_settings: ::core::option::Option<DotnetSettings>,
    /// Settings for Ruby client libraries.
    #[prost(message, optional, tag = "27")]
    pub ruby_settings: ::core::option::Option<RubySettings>,
    /// Settings for Go client libraries.
    #[prost(message, optional, tag = "28")]
    pub go_settings: ::core::option::Option<GoSettings>,
}
/// This message configures the settings for publishing [Google Cloud Client
/// libraries](<https://cloud.google.com/apis/docs/cloud-client-libraries>)
/// generated from the service config.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Publishing {
    /// A list of API method settings, e.g. the behavior for methods that use the
    /// long-running operation pattern.
    #[prost(message, repeated, tag = "2")]
    pub method_settings: ::prost::alloc::vec::Vec<MethodSettings>,
    /// Link to a place that API users can report issues.  Example:
    /// <https://issuetracker.google.com/issues/new?component=190865&template=1161103>
    #[prost(string, tag = "101")]
    pub new_issue_uri: ::prost::alloc::string::String,
    /// Link to product home page.  Example:
    /// <https://cloud.google.com/asset-inventory/docs/overview>
    #[prost(string, tag = "102")]
    pub documentation_uri: ::prost::alloc::string::String,
    /// Used as a tracking tag when collecting data about the APIs developer
    /// relations artifacts like docs, packages delivered to package managers,
    /// etc.  Example: "speech".
    #[prost(string, tag = "103")]
    pub api_short_name: ::prost::alloc::string::String,
    /// GitHub label to apply to issues and pull requests opened for this API.
    #[prost(string, tag = "104")]
    pub github_label: ::prost::alloc::string::String,
    /// GitHub teams to be added to CODEOWNERS in the directory in GitHub
    /// containing source code for the client libraries for this API.
    #[prost(string, repeated, tag = "105")]
    pub codeowner_github_teams: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// A prefix used in sample code when demarking regions to be included in
    /// documentation.
    #[prost(string, tag = "106")]
    pub doc_tag_prefix: ::prost::alloc::string::String,
    /// For whom the client library is being published.
    #[prost(enumeration = "ClientLibraryOrganization", tag = "107")]
    pub organization: i32,
    /// Client library settings.  If the same version string appears multiple
    /// times in this list, then the last one wins.  Settings from earlier
    /// settings with the same version string are discarded.
    #[prost(message, repeated, tag = "109")]
    pub library_settings: ::prost::alloc::vec::Vec<ClientLibrarySettings>,
}
/// Settings for Java client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JavaSettings {
    /// The package name to use in Java. Clobbers the java_package option
    /// set in the protobuf. This should be used **only** by APIs
    /// who have already set the language_settings.java.package_name" field
    /// in gapic.yaml. API teams should use the protobuf java_package option
    /// where possible.
    ///
    /// Example of a YAML configuration::
    ///
    ///   publishing:
    ///     java_settings:
    ///       library_package: com.google.cloud.pubsub.v1
    #[prost(string, tag = "1")]
    pub library_package: ::prost::alloc::string::String,
    /// Configure the Java class name to use instead of the service's for its
    /// corresponding generated GAPIC client. Keys are fully-qualified
    /// service names as they appear in the protobuf (including the full
    /// the language_settings.java.interface_names" field in gapic.yaml. API
    /// teams should otherwise use the service name as it appears in the
    /// protobuf.
    ///
    /// Example of a YAML configuration::
    ///
    ///   publishing:
    ///     java_settings:
    ///       service_class_names:
    ///         - google.pubsub.v1.Publisher: TopicAdmin
    ///         - google.pubsub.v1.Subscriber: SubscriptionAdmin
    #[prost(map = "string, string", tag = "2")]
    pub service_class_names:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Some settings.
    #[prost(message, optional, tag = "3")]
    pub common: ::core::option::Option<CommonLanguageSettings>,
}
/// Settings for C++ client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CppSettings {
    /// Some settings.
    #[prost(message, optional, tag = "1")]
    pub common: ::core::option::Option<CommonLanguageSettings>,
}
/// Settings for Php client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PhpSettings {
    /// Some settings.
    #[prost(message, optional, tag = "1")]
    pub common: ::core::option::Option<CommonLanguageSettings>,
}
/// Settings for Python client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PythonSettings {
    /// Some settings.
    #[prost(message, optional, tag = "1")]
    pub common: ::core::option::Option<CommonLanguageSettings>,
}
/// Settings for Node client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeSettings {
    /// Some settings.
    #[prost(message, optional, tag = "1")]
    pub common: ::core::option::Option<CommonLanguageSettings>,
}
/// Settings for Dotnet client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DotnetSettings {
    /// Some settings.
    #[prost(message, optional, tag = "1")]
    pub common: ::core::option::Option<CommonLanguageSettings>,
}
/// Settings for Ruby client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RubySettings {
    /// Some settings.
    #[prost(message, optional, tag = "1")]
    pub common: ::core::option::Option<CommonLanguageSettings>,
}
/// Settings for Go client libraries.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GoSettings {
    /// Some settings.
    #[prost(message, optional, tag = "1")]
    pub common: ::core::option::Option<CommonLanguageSettings>,
}
/// Describes the generator configuration for a method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MethodSettings {
    /// The fully qualified name of the method, for which the options below apply.
    /// This is used to find the method to apply the options.
    #[prost(string, tag = "1")]
    pub selector: ::prost::alloc::string::String,
    /// Describes settings to use for long-running operations when generating
    /// API methods for RPCs. Complements RPCs that use the annotations in
    /// google/longrunning/operations.proto.
    ///
    /// Example of a YAML configuration::
    ///
    ///   publishing:
    ///     method_behavior:
    ///       - selector: CreateAdDomain
    ///         long_running:
    ///           initial_poll_delay:
    ///             seconds: 60 # 1 minute
    ///           poll_delay_multiplier: 1.5
    ///           max_poll_delay:
    ///             seconds: 360 # 6 minutes
    ///           total_poll_timeout:
    ///              seconds: 54000 # 90 minutes
    #[prost(message, optional, tag = "2")]
    pub long_running: ::core::option::Option<method_settings::LongRunning>,
}
/// Nested message and enum types in `MethodSettings`.
pub mod method_settings {
    /// Describes settings to use when generating API methods that use the
    /// long-running operation pattern.
    /// All default values below are from those used in the client library
    /// generators (e.g.
    /// \[Java\](<https://github.com/googleapis/gapic-generator-java/blob/04c2faa191a9b5a10b92392fe8482279c4404803/src/main/java/com/google/api/generator/gapic/composer/common/RetrySettingsComposer.java>)).
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct LongRunning {
        /// Initial delay after which the first poll request will be made.
        /// Default value: 5 seconds.
        #[prost(message, optional, tag = "1")]
        pub initial_poll_delay: ::core::option::Option<::prost_types::Duration>,
        /// Multiplier to gradually increase delay between subsequent polls until it
        /// reaches max_poll_delay.
        /// Default value: 1.5.
        #[prost(float, tag = "2")]
        pub poll_delay_multiplier: f32,
        /// Maximum time between two subsequent poll requests.
        /// Default value: 45 seconds.
        #[prost(message, optional, tag = "3")]
        pub max_poll_delay: ::core::option::Option<::prost_types::Duration>,
        /// Total polling timeout.
        /// Default value: 5 minutes.
        #[prost(message, optional, tag = "4")]
        pub total_poll_timeout: ::core::option::Option<::prost_types::Duration>,
    }
}
/// The organization for which the client libraries are being published.
/// Affects the url where generated docs are published, etc.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ClientLibraryOrganization {
    /// Not useful.
    Unspecified = 0,
    /// Google Cloud Platform Org.
    Cloud = 1,
    /// Ads (Advertising) Org.
    Ads = 2,
    /// Photos Org.
    Photos = 3,
    /// Street View Org.
    StreetView = 4,
}
impl ClientLibraryOrganization {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ClientLibraryOrganization::Unspecified => "CLIENT_LIBRARY_ORGANIZATION_UNSPECIFIED",
            ClientLibraryOrganization::Cloud => "CLOUD",
            ClientLibraryOrganization::Ads => "ADS",
            ClientLibraryOrganization::Photos => "PHOTOS",
            ClientLibraryOrganization::StreetView => "STREET_VIEW",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CLIENT_LIBRARY_ORGANIZATION_UNSPECIFIED" => Some(Self::Unspecified),
            "CLOUD" => Some(Self::Cloud),
            "ADS" => Some(Self::Ads),
            "PHOTOS" => Some(Self::Photos),
            "STREET_VIEW" => Some(Self::StreetView),
            _ => None,
        }
    }
}
/// To where should client libraries be published?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ClientLibraryDestination {
    /// Client libraries will neither be generated nor published to package
    /// managers.
    Unspecified = 0,
    /// Generate the client library in a repo under github.com/googleapis,
    /// but don't publish it to package managers.
    Github = 10,
    /// Publish the library to package managers like nuget.org and npmjs.com.
    PackageManager = 20,
}
impl ClientLibraryDestination {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ClientLibraryDestination::Unspecified => "CLIENT_LIBRARY_DESTINATION_UNSPECIFIED",
            ClientLibraryDestination::Github => "GITHUB",
            ClientLibraryDestination::PackageManager => "PACKAGE_MANAGER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CLIENT_LIBRARY_DESTINATION_UNSPECIFIED" => Some(Self::Unspecified),
            "GITHUB" => Some(Self::Github),
            "PACKAGE_MANAGER" => Some(Self::PackageManager),
            _ => None,
        }
    }
}
/// An indicator of the behavior of a given field (for example, that a field
/// is required in requests, or given as output but ignored as input).
/// This **does not** change the behavior in protocol buffers itself; it only
/// denotes the behavior and may affect how API tooling handles the field.
///
/// Note: This enum **may** receive new values in the future.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FieldBehavior {
    /// Conventional default for enums. Do not use this.
    Unspecified = 0,
    /// Specifically denotes a field as optional.
    /// While all fields in protocol buffers are optional, this may be specified
    /// for emphasis if appropriate.
    Optional = 1,
    /// Denotes a field as required.
    /// This indicates that the field **must** be provided as part of the request,
    /// and failure to do so will cause an error (usually `INVALID_ARGUMENT`).
    Required = 2,
    /// Denotes a field as output only.
    /// This indicates that the field is provided in responses, but including the
    /// field in a request does nothing (the server *must* ignore it and
    /// *must not* throw an error as a result of the field's presence).
    OutputOnly = 3,
    /// Denotes a field as input only.
    /// This indicates that the field is provided in requests, and the
    /// corresponding field is not included in output.
    InputOnly = 4,
    /// Denotes a field as immutable.
    /// This indicates that the field may be set once in a request to create a
    /// resource, but may not be changed thereafter.
    Immutable = 5,
    /// Denotes that a (repeated) field is an unordered list.
    /// This indicates that the service may provide the elements of the list
    /// in any arbitrary  order, rather than the order the user originally
    /// provided. Additionally, the list's order may or may not be stable.
    UnorderedList = 6,
    /// Denotes that this field returns a non-empty default value if not set.
    /// This indicates that if the user provides the empty value in a request,
    /// a non-empty value will be returned. The user will not be aware of what
    /// non-empty value to expect.
    NonEmptyDefault = 7,
}
impl FieldBehavior {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            FieldBehavior::Unspecified => "FIELD_BEHAVIOR_UNSPECIFIED",
            FieldBehavior::Optional => "OPTIONAL",
            FieldBehavior::Required => "REQUIRED",
            FieldBehavior::OutputOnly => "OUTPUT_ONLY",
            FieldBehavior::InputOnly => "INPUT_ONLY",
            FieldBehavior::Immutable => "IMMUTABLE",
            FieldBehavior::UnorderedList => "UNORDERED_LIST",
            FieldBehavior::NonEmptyDefault => "NON_EMPTY_DEFAULT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FIELD_BEHAVIOR_UNSPECIFIED" => Some(Self::Unspecified),
            "OPTIONAL" => Some(Self::Optional),
            "REQUIRED" => Some(Self::Required),
            "OUTPUT_ONLY" => Some(Self::OutputOnly),
            "INPUT_ONLY" => Some(Self::InputOnly),
            "IMMUTABLE" => Some(Self::Immutable),
            "UNORDERED_LIST" => Some(Self::UnorderedList),
            "NON_EMPTY_DEFAULT" => Some(Self::NonEmptyDefault),
            _ => None,
        }
    }
}
/// A simple descriptor of a resource type.
///
/// ResourceDescriptor annotates a resource message (either by means of a
/// protobuf annotation or use in the service config), and associates the
/// resource's schema, the resource type, and the pattern of the resource name.
///
/// Example:
///
///      message Topic {
///        // Indicates this message defines a resource schema.
///        // Declares the resource type in the format of {service}/{kind}.
///        // For Kubernetes resources, the format is {api group}/{kind}.
///        option (google.api.resource) = {
///          type: "pubsub.googleapis.com/Topic"
///          pattern: "projects/{project}/topics/{topic}"
///        };
///      }
///
/// The ResourceDescriptor Yaml config will look like:
///
///      resources:
///      - type: "pubsub.googleapis.com/Topic"
///        pattern: "projects/{project}/topics/{topic}"
///
/// Sometimes, resources have multiple patterns, typically because they can
/// live under multiple parents.
///
/// Example:
///
///      message LogEntry {
///        option (google.api.resource) = {
///          type: "logging.googleapis.com/LogEntry"
///          pattern: "projects/{project}/logs/{log}"
///          pattern: "folders/{folder}/logs/{log}"
///          pattern: "organizations/{organization}/logs/{log}"
///          pattern: "billingAccounts/{billing_account}/logs/{log}"
///        };
///      }
///
/// The ResourceDescriptor Yaml config will look like:
///
///      resources:
///      - type: 'logging.googleapis.com/LogEntry'
///        pattern: "projects/{project}/logs/{log}"
///        pattern: "folders/{folder}/logs/{log}"
///        pattern: "organizations/{organization}/logs/{log}"
///        pattern: "billingAccounts/{billing_account}/logs/{log}"
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceDescriptor {
    /// The resource type. It must be in the format of
    /// {service_name}/{resource_type_kind}. The `resource_type_kind` must be
    /// singular and must not include version numbers.
    ///
    /// Example: `storage.googleapis.com/Bucket`
    ///
    /// The value of the resource_type_kind must follow the regular expression
    /// /\[A-Za-z][a-zA-Z0-9\]+/. It should start with an upper case character and
    /// should use PascalCase (UpperCamelCase). The maximum number of
    /// characters allowed for the `resource_type_kind` is 100.
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    /// Optional. The relative resource name pattern associated with this resource
    /// type. The DNS prefix of the full resource name shouldn't be specified here.
    ///
    /// The path pattern must follow the syntax, which aligns with HTTP binding
    /// syntax:
    ///
    ///      Template = Segment { "/" Segment } ;
    ///      Segment = LITERAL | Variable ;
    ///      Variable = "{" LITERAL "}" ;
    ///
    /// Examples:
    ///
    ///      - "projects/{project}/topics/{topic}"
    ///      - "projects/{project}/knowledgeBases/{knowledge_base}"
    ///
    /// The components in braces correspond to the IDs for each resource in the
    /// hierarchy. It is expected that, if multiple patterns are provided,
    /// the same component name (e.g. "project") refers to IDs of the same
    /// type of resource.
    #[prost(string, repeated, tag = "2")]
    pub pattern: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Optional. The field on the resource that designates the resource name
    /// field. If omitted, this is assumed to be "name".
    #[prost(string, tag = "3")]
    pub name_field: ::prost::alloc::string::String,
    /// Optional. The historical or future-looking state of the resource pattern.
    ///
    /// Example:
    ///
    ///      // The InspectTemplate message originally only supported resource
    ///      // names with organization, and project was added later.
    ///      message InspectTemplate {
    ///        option (google.api.resource) = {
    ///          type: "dlp.googleapis.com/InspectTemplate"
    ///          pattern:
    ///          "organizations/{organization}/inspectTemplates/{inspect_template}"
    ///          pattern: "projects/{project}/inspectTemplates/{inspect_template}"
    ///          history: ORIGINALLY_SINGLE_PATTERN
    ///        };
    ///      }
    #[prost(enumeration = "resource_descriptor::History", tag = "4")]
    pub history: i32,
    /// The plural name used in the resource name and permission names, such as
    /// 'projects' for the resource name of 'projects/{project}' and the permission
    /// name of 'cloudresourcemanager.googleapis.com/projects.get'. It is the same
    /// concept of the `plural` field in k8s CRD spec
    /// <https://kubernetes.io/docs/tasks/access-kubernetes-api/custom-resources/custom-resource-definitions/>
    ///
    /// Note: The plural form is required even for singleton resources. See
    /// <https://aip.dev/156>
    #[prost(string, tag = "5")]
    pub plural: ::prost::alloc::string::String,
    /// The same concept of the `singular` field in k8s CRD spec
    /// <https://kubernetes.io/docs/tasks/access-kubernetes-api/custom-resources/custom-resource-definitions/>
    /// Such as "project" for the `resourcemanager.googleapis.com/Project` type.
    #[prost(string, tag = "6")]
    pub singular: ::prost::alloc::string::String,
    /// Style flag(s) for this resource.
    /// These indicate that a resource is expected to conform to a given
    /// style. See the specific style flags for additional information.
    #[prost(enumeration = "resource_descriptor::Style", repeated, tag = "10")]
    pub style: ::prost::alloc::vec::Vec<i32>,
}
/// Nested message and enum types in `ResourceDescriptor`.
pub mod resource_descriptor {
    /// A description of the historical or future-looking state of the
    /// resource pattern.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum History {
        /// The "unset" value.
        Unspecified = 0,
        /// The resource originally had one pattern and launched as such, and
        /// additional patterns were added later.
        OriginallySinglePattern = 1,
        /// The resource has one pattern, but the API owner expects to add more
        /// later. (This is the inverse of ORIGINALLY_SINGLE_PATTERN, and prevents
        /// that from being necessary once there are multiple patterns.)
        FutureMultiPattern = 2,
    }
    impl History {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                History::Unspecified => "HISTORY_UNSPECIFIED",
                History::OriginallySinglePattern => "ORIGINALLY_SINGLE_PATTERN",
                History::FutureMultiPattern => "FUTURE_MULTI_PATTERN",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "HISTORY_UNSPECIFIED" => Some(Self::Unspecified),
                "ORIGINALLY_SINGLE_PATTERN" => Some(Self::OriginallySinglePattern),
                "FUTURE_MULTI_PATTERN" => Some(Self::FutureMultiPattern),
                _ => None,
            }
        }
    }
    /// A flag representing a specific style that a resource claims to conform to.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Style {
        /// The unspecified value. Do not use.
        Unspecified = 0,
        /// This resource is intended to be "declarative-friendly".
        ///
        /// Declarative-friendly resources must be more strictly consistent, and
        /// setting this to true communicates to tools that this resource should
        /// adhere to declarative-friendly expectations.
        ///
        /// Note: This is used by the API linter (linter.aip.dev) to enable
        /// additional checks.
        DeclarativeFriendly = 1,
    }
    impl Style {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Style::Unspecified => "STYLE_UNSPECIFIED",
                Style::DeclarativeFriendly => "DECLARATIVE_FRIENDLY",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STYLE_UNSPECIFIED" => Some(Self::Unspecified),
                "DECLARATIVE_FRIENDLY" => Some(Self::DeclarativeFriendly),
                _ => None,
            }
        }
    }
}
/// Defines a proto annotation that describes a string field that refers to
/// an API resource.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceReference {
    /// The resource type that the annotated field references.
    ///
    /// Example:
    ///
    ///      message Subscription {
    ///        string topic = 2 [(google.api.resource_reference) = {
    ///          type: "pubsub.googleapis.com/Topic"
    ///        }];
    ///      }
    ///
    /// Occasionally, a field may reference an arbitrary resource. In this case,
    /// APIs use the special value * in their resource reference.
    ///
    /// Example:
    ///
    ///      message GetIamPolicyRequest {
    ///        string resource = 2 [(google.api.resource_reference) = {
    ///          type: "*"
    ///        }];
    ///      }
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    /// The resource type of a child collection that the annotated field
    /// references. This is useful for annotating the `parent` field that
    /// doesn't have a fixed resource type.
    ///
    /// Example:
    ///
    ///      message ListLogEntriesRequest {
    ///        string parent = 1 [(google.api.resource_reference) = {
    ///          child_type: "logging.googleapis.com/LogEntry"
    ///        };
    ///      }
    #[prost(string, tag = "2")]
    pub child_type: ::prost::alloc::string::String,
}
/// A description of a label.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelDescriptor {
    /// The label key.
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    /// The type of data that can be assigned to the label.
    #[prost(enumeration = "label_descriptor::ValueType", tag = "2")]
    pub value_type: i32,
    /// A human-readable description for the label.
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
}
/// Nested message and enum types in `LabelDescriptor`.
pub mod label_descriptor {
    /// Value types that can be used as label values.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum ValueType {
        /// A variable-length string. This is the default.
        String = 0,
        /// Boolean; true or false.
        Bool = 1,
        /// A 64-bit signed integer.
        Int64 = 2,
    }
    impl ValueType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ValueType::String => "STRING",
                ValueType::Bool => "BOOL",
                ValueType::Int64 => "INT64",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STRING" => Some(Self::String),
                "BOOL" => Some(Self::Bool),
                "INT64" => Some(Self::Int64),
                _ => None,
            }
        }
    }
}
/// An object that describes the schema of a \[MonitoredResource][google.api.MonitoredResource\] object using a
/// type name and a set of labels.  For example, the monitored resource
/// descriptor for Google Compute Engine VM instances has a type of
/// `"gce_instance"` and specifies the use of the labels `"instance_id"` and
/// `"zone"` to identify particular VM instances.
///
/// Different APIs can support different monitored resource types. APIs generally
/// provide a `list` method that returns the monitored resource descriptors used
/// by the API.
///
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MonitoredResourceDescriptor {
    /// Optional. The resource name of the monitored resource descriptor:
    /// `"projects/{project_id}/monitoredResourceDescriptors/{type}"` where
    /// {type} is the value of the `type` field in this object and
    /// {project_id} is a project ID that provides API-specific context for
    /// accessing the type.  APIs that do not use project information can use the
    /// resource name format `"monitoredResourceDescriptors/{type}"`.
    #[prost(string, tag = "5")]
    pub name: ::prost::alloc::string::String,
    /// Required. The monitored resource type. For example, the type
    /// `"cloudsql_database"` represents databases in Google Cloud SQL.
    ///   For a list of types, see [Monitoring resource
    ///   types](<https://cloud.google.com/monitoring/api/resources>)
    /// and [Logging resource
    /// types](<https://cloud.google.com/logging/docs/api/v2/resource-list>).
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    /// Optional. A concise name for the monitored resource type that might be
    /// displayed in user interfaces. It should be a Title Cased Noun Phrase,
    /// without any article or other determiners. For example,
    /// `"Google Cloud SQL Database"`.
    #[prost(string, tag = "2")]
    pub display_name: ::prost::alloc::string::String,
    /// Optional. A detailed description of the monitored resource type that might
    /// be used in documentation.
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    /// Required. A set of labels used to describe instances of this monitored
    /// resource type. For example, an individual Google Cloud SQL database is
    /// identified by values for the labels `"database_id"` and `"zone"`.
    #[prost(message, repeated, tag = "4")]
    pub labels: ::prost::alloc::vec::Vec<LabelDescriptor>,
    /// Optional. The launch stage of the monitored resource definition.
    #[prost(enumeration = "LaunchStage", tag = "7")]
    pub launch_stage: i32,
}
/// An object representing a resource that can be used for monitoring, logging,
/// billing, or other purposes. Examples include virtual machine instances,
/// databases, and storage devices such as disks. The `type` field identifies a
/// \[MonitoredResourceDescriptor][google.api.MonitoredResourceDescriptor\] object that describes the resource's
/// schema. Information in the `labels` field identifies the actual resource and
/// its attributes according to the schema. For example, a particular Compute
/// Engine VM instance could be represented by the following object, because the
/// \[MonitoredResourceDescriptor][google.api.MonitoredResourceDescriptor\] for `"gce_instance"` has labels
/// `"project_id"`, `"instance_id"` and `"zone"`:
///
///      { "type": "gce_instance",
///        "labels": { "project_id": "my-project",
///                    "instance_id": "12345678901234",
///                    "zone": "us-central1-a" }}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MonitoredResource {
    /// Required. The monitored resource type. This field must match
    /// the `type` field of a \[MonitoredResourceDescriptor][google.api.MonitoredResourceDescriptor\] object. For
    /// example, the type of a Compute Engine VM instance is `gce_instance`.
    /// Some descriptors include the service name in the type; for example,
    /// the type of a Datastream stream is `datastream.googleapis.com/Stream`.
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    /// Required. Values for all of the labels listed in the associated monitored
    /// resource descriptor. For example, Compute Engine VM instances use the
    /// labels `"project_id"`, `"instance_id"`, and `"zone"`.
    #[prost(map = "string, string", tag = "2")]
    pub labels:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
/// Auxiliary metadata for a \[MonitoredResource][google.api.MonitoredResource\] object.
/// \[MonitoredResource][google.api.MonitoredResource\] objects contain the minimum set of information to
/// uniquely identify a monitored resource instance. There is some other useful
/// auxiliary metadata. Monitoring and Logging use an ingestion
/// pipeline to extract metadata for cloud resources of all types, and store
/// the metadata in this message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MonitoredResourceMetadata {
    /// Output only. Values for predefined system metadata labels.
    /// System labels are a kind of metadata extracted by Google, including
    /// "machine_image", "vpc", "subnet_id",
    /// "security_group", "name", etc.
    /// System label values can be only strings, Boolean values, or a list of
    /// strings. For example:
    ///
    ///      { "name": "my-test-instance",
    ///        "security_group": ["a", "b", "c"],
    ///        "spot_instance": false }
    #[prost(message, optional, tag = "1")]
    pub system_labels: ::core::option::Option<::prost_types::Struct>,
    /// Output only. A map of user-defined metadata labels.
    #[prost(map = "string, string", tag = "2")]
    pub user_labels:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
