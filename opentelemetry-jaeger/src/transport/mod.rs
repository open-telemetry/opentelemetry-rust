//! Additional Thrift transport implementations
mod noop;

pub(crate) use noop::TNoopChannel;
