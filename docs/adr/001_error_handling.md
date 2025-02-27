# Error handling patterns in public API interfaces
## Date
27 Feb 2025 

## Summary

This ADR describes the general pattern we will follow when modelling errors in public API interfaces - that is, APIs that are exposed to users of the project's published crates. . It summarises the discussion and final option from [#2571](https://github.com/open-telemetry/opentelemetry-rust/issues/2571); for more context check out that issue. 

We will focus on the exporter traits in this example, but the outcome should be applied to _all_ public traits and their fallible operations. 

These include [SpanExporter](https://github.com/open-telemetry/opentelemetry-rust/blob/eca1ce87084c39667061281e662d5edb9a002882/opentelemetry-sdk/src/trace/export.rs#L18), [LogExporter](https://github.com/open-telemetry/opentelemetry-rust/blob/eca1ce87084c39667061281e662d5edb9a002882/opentelemetry-sdk/src/logs/export.rs#L115), and [PushMetricExporter](https://github.com/open-telemetry/opentelemetry-rust/blob/eca1ce87084c39667061281e662d5edb9a002882/opentelemetry-sdk/src/metrics/exporter.rs#L11) which form part of the API surface of `opentelemetry-sdk`.

There are various ways to handle errors on trait methods, including swallowing them and logging, panicing, returning a shared global error, or returning a method-specific error. We strive for consistency, and we want to be sure that we've put enough thought into what this looks like that we don't have to make breaking interface changes unecessarily in the future.

## Design Guidance

### 1. Panics are only appropriate at startup
Failures _after_ startup should not panic, instead returning errors to the caller where appropriate, _or_ logging an error if not appropriate.
Some of the opentelemetry SDK interfaces are dictated by the specification in way such that they may not return errors. 

### 2. Consolidate error types within a trait where we can, let them diverge when we can't**

We aim to consolidate error types where possible _without indicating a function may return more errors than it can actually return_. 
Here's an example:

```rust
enum ErrorOne {
  TooBig,
  TooSmall,
}

enum ErrorTwo {
  TooLong,
  TooShort
}

trait MyTrait {
  fn action_one() -> Result<(), ErrorOne>;

  // Action two and three share the same error type. 
  // We do not introduce a common error MyTraitError for all operations, as this would
  // force all methods on the trait to indicate they return errors they do not return,
  // complicating things for the caller.  
  fn action_two() -> Result<(), ErrorTwo>;
  fn action_three() -> Result<(), ErrorTwo>;
}
```
## 3. Consolidate error types between signals where we can, let them diverge where we can't

Consider the `Exporter`s mentioned earlier. Each of them has the same failure indicators - as dicated by the OpenTelemetry spec  - and we will
share the error types accordingly: 

```rust

/// opentelemetry-sdk::error

#[derive(Error, Debug)]
pub enum OTelSdkError {
    #[error("Shutdown already invoked")]
    AlreadyShutdown,
    
    #[error("Operation failed: {0}")]
    InternalFailure(String),

    /** ... additional errors ... **/ 
}

pub type OTelSdkResult = Result<(), OTelSdkError>;

/// signal-specific exporter traits all share the same 
/// result types for the exporter operations.

// pub trait LogExporter {
// pub trait SpanExporter {
pub trait PushMetricExporter {

    fn export(&self, /* ... */) -> OtelSdkResult;
    fn force_flush(&self, /* ... */ ) -> OTelSdkResult;
    fn shutdown(&self, /* ... */ ) -> OTelSdkResult;
```

If this were _not_ the case - if we needed to mark an extra error for instance for `LogExporter` that the caller could reasonably handle - 
we would let that error traits diverge at that point. 

### 4. Box custom errors where a savvy caller may be able to handle them, stringify them if not

Note above that we do not box anything into `InternalFailure`. Our rule here is that if the caller cannot reasonably be expected to handle a particular error variant, we will use a simplified interface that returns only a descriptive string. In the concrete example we are using with the exporters, we have a [strong signal in the opentelemetry-specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/sdk.md#export) that indicates that the error types _are not actionable_ by the caller. 

If the caller may potentially recover from an error, we will follow the generally-accepted best practice (e.g., see [canonical's guide](https://canonical.github.io/rust-best-practices/error-and-panic-discipline.html) and instead preserve the nested error:

```rust
#[derive(Debug, Error)]
pub enum MyError {
    #[error("Error one occurred")]
    ErrorOne, 

    #[error("Operation failed: {source}")]
    OtherError {
        #[from]
        source: Box<dyn Error + Send + Sync>,
    },
}
```

