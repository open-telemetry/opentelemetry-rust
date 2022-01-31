// This file is generated by rust-protobuf 2.26.0. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `opentelemetry/proto/collector/trace/v1/trace_service.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_26_0;

#[derive(PartialEq,Clone,Default)]
#[cfg_attr(feature = "with-serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ExportTraceServiceRequest {
    // message fields
    pub resource_spans: ::protobuf::RepeatedField<super::trace::ResourceSpans>,
    // special fields
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub unknown_fields: ::protobuf::UnknownFields,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a ExportTraceServiceRequest {
    fn default() -> &'a ExportTraceServiceRequest {
        <ExportTraceServiceRequest as ::protobuf::Message>::default_instance()
    }
}

impl ExportTraceServiceRequest {
    pub fn new() -> ExportTraceServiceRequest {
        ::std::default::Default::default()
    }

    // repeated .opentelemetry.proto.trace.v1.ResourceSpans resource_spans = 1;


    pub fn get_resource_spans(&self) -> &[super::trace::ResourceSpans] {
        &self.resource_spans
    }
    pub fn clear_resource_spans(&mut self) {
        self.resource_spans.clear();
    }

    // Param is passed by value, moved
    pub fn set_resource_spans(&mut self, v: ::protobuf::RepeatedField<super::trace::ResourceSpans>) {
        self.resource_spans = v;
    }

    // Mutable pointer to the field.
    pub fn mut_resource_spans(&mut self) -> &mut ::protobuf::RepeatedField<super::trace::ResourceSpans> {
        &mut self.resource_spans
    }

    // Take field
    pub fn take_resource_spans(&mut self) -> ::protobuf::RepeatedField<super::trace::ResourceSpans> {
        ::std::mem::replace(&mut self.resource_spans, ::protobuf::RepeatedField::new())
    }
}

impl ::protobuf::Message for ExportTraceServiceRequest {
    fn is_initialized(&self) -> bool {
        for v in &self.resource_spans {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.resource_spans)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.resource_spans {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        for v in &self.resource_spans {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::trace::ResourceSpans>>(
                "resource_spans",
                |m: &ExportTraceServiceRequest| { &m.resource_spans },
                |m: &mut ExportTraceServiceRequest| { &mut m.resource_spans },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<ExportTraceServiceRequest>(
                "ExportTraceServiceRequest",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static ExportTraceServiceRequest {
        static instance: ::protobuf::rt::LazyV2<ExportTraceServiceRequest> = ::protobuf::rt::LazyV2::INIT;
        instance.get(ExportTraceServiceRequest::new)
    }
}

impl ::protobuf::Clear for ExportTraceServiceRequest {
    fn clear(&mut self) {
        self.resource_spans.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ExportTraceServiceRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ExportTraceServiceRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
#[cfg_attr(feature = "with-serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ExportTraceServiceResponse {
    // special fields
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub unknown_fields: ::protobuf::UnknownFields,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a ExportTraceServiceResponse {
    fn default() -> &'a ExportTraceServiceResponse {
        <ExportTraceServiceResponse as ::protobuf::Message>::default_instance()
    }
}

impl ExportTraceServiceResponse {
    pub fn new() -> ExportTraceServiceResponse {
        ::std::default::Default::default()
    }
}

impl ::protobuf::Message for ExportTraceServiceResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> ExportTraceServiceResponse {
        ExportTraceServiceResponse::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let fields = ::std::vec::Vec::new();
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<ExportTraceServiceResponse>(
                "ExportTraceServiceResponse",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static ExportTraceServiceResponse {
        static instance: ::protobuf::rt::LazyV2<ExportTraceServiceResponse> = ::protobuf::rt::LazyV2::INIT;
        instance.get(ExportTraceServiceResponse::new)
    }
}

impl ::protobuf::Clear for ExportTraceServiceResponse {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ExportTraceServiceResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ExportTraceServiceResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n:opentelemetry/proto/collector/trace/v1/trace_service.proto\x12&opente\
    lemetry.proto.collector.trace.v1\x1a(opentelemetry/proto/trace/v1/trace.\
    proto\"o\n\x19ExportTraceServiceRequest\x12R\n\x0eresource_spans\x18\x01\
    \x20\x03(\x0b2+.opentelemetry.proto.trace.v1.ResourceSpansR\rresourceSpa\
    ns\"\x1c\n\x1aExportTraceServiceResponse2\xa2\x01\n\x0cTraceService\x12\
    \x91\x01\n\x06Export\x12A.opentelemetry.proto.collector.trace.v1.ExportT\
    raceServiceRequest\x1aB.opentelemetry.proto.collector.trace.v1.ExportTra\
    ceServiceResponse\"\0B\x89\x01\n)io.opentelemetry.proto.collector.trace.\
    v1B\x11TraceServiceProtoP\x01ZGgithub.com/open-telemetry/opentelemetry-p\
    roto/gen/go/collector/trace/v1b\x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
