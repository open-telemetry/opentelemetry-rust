# Error handling patterns in public API interfaces

## Date
17 Feb 2025 

## Accepted Option

**Option 3** 

## Context and Problem Statement
There is uncertainty around how to model errors in the `opentelemetry-rust` public API interfaces - that is, APIs that are exposed to users of the project's published crates. This is for example the case with the exporter traits - [SpanExporter](https://github.com/open-telemetry/opentelemetry-rust/blob/eca1ce87084c39667061281e662d5edb9a002882/opentelemetry-sdk/src/trace/export.rs#L18), [LogExporter](https://github.com/open-telemetry/opentelemetry-rust/blob/eca1ce87084c39667061281e662d5edb9a002882/opentelemetry-sdk/src/logs/export.rs#L115), and [PushMetricExporter](https://github.com/open-telemetry/opentelemetry-rust/blob/eca1ce87084c39667061281e662d5edb9a002882/opentelemetry-sdk/src/metrics/exporter.rs#L11) which form part of the API surface of `opentelemetry-sdk`.

We will focus on the exporter traits in this example, but the outcome should be applied to _all_ public traits and their fallible operations. 

There are various ways to handle errors on trait methods, including swallowing them and logging, panicing, returning a shared global error, or returning a method-specific error. We strive for consistency, and we want to be sure that we've put enough thought into what this looks like that we don't have to make breaking interface changes unecessarily in the future.

This was discussed extensively in #2571.


## Related Work

* #2564 
* #2561 
* #2381

## Considered Options

**Option 1: Continue as is**
Continue the status quo, returning a mix of either nothing or the trait-wide error type. This is inconsistent and limits the caller's ability to handle errors. 

**Option 2: Extend trait-wide error type to all methods on trait**
In this option we have an error type per trait regardless of the potential error paths for the individual methods. Concretely if `fn (a)` can return `Failure1` and `Failure2`,  and `fn (b)` can return `FailureC`, we have a unified error type that contains `Failure`, `Failure2`, and `Failure3`.

 This will mean that callers will have to know how and when to discard errors from a particular API call based on an understanding of which subset of errors that particular call can make. 

Conversely, it will reduce the number of error types in the code base. 

**Option 3: Use shared errors where practical across signals, devolve into individual errors per operation if they need to diverge**

Here we aim to consolidate error types where possible _without indicating a function may return more errors than it can actually return_. Conversely in **Option 2**, a caller of either of the example functions is forced to handle or discard all errors. In this case, we choose to sacrifice the single error and diverge into a separate error for each operation.

Our preference for error types is thus:

1. Consolidated error that covers all methods of a particular "trait type" (e.g., signal export) and method
1. Devolves into error type per method of a particular trait type (e.g., `SdkShutdownResult`, `SdkExportResult`) _if the error types need to diverge_
1. May alternatively devolve into error type per signal (e.g., `SpanExporter`) if the _signals diverge_

This approach generalises across both **signals** and **trait methods**. For example, returning to our exporter traits, we have a trait that looks the same for each signal, with the same three methods. Upon closer inspection (#2600), the potential error set is the same both between the methods *and* between the signals; this means we can use a single shared error type across both axes:

```rust

#[derive(Error, Debug)]

// Errors that can occur during SDK operations export(), force_flush() and shutdown().
pub enum OTelSdkError {

    /// All error types in here may be returned by any of the operations
    /// of the exporters, on any of their APIs.
    /// If this were not the case, we would not be able to use a shared error.
    #[error("Shutdown already invoked")]
    AlreadyShutdown,

    // ... Other errors ...

    /// The error message is intended for logging purposes only and should not
    /// be used to make programmatic decisions. It is implementation-specific
    /// and subject to change without notice. Consumers of this error should not
    /// rely on its content beyond logging.
    #[error("Operation failed: {0}")]
    InternalFailure(String),
}

pub type OTelSdkResult = Result<(), OTelSdkError>;
```

... which the traits themselves use:

```rust

//
// The actionable errors returned by the exporter traits are effectively
// the same for all operations; we can use a single shared error.
// 

use opentelemetry_sdk::error::OTelSdkResult;

pub trait LogExporter {
	fn export(...) -> OtelSdkResult;
	fn shutdown(...) -> OtelSdkResult; 
  fn force_flush(...) -> OTelSdkResult;
}

// ...

pub trait SpanExporter {
	fn export(...) -> OtelSdkResult;
	fn shutdown(...) -> OtelSdkResult;
  fn force_flush(...) -> OTelSdkResult;
}

```

### When to box custom errors

Note above that we do not box anything into `InternalFailure`. Our rule here is that if the caller cannot reasonably be expected to handle a particular error variant, we will use a simplified interface that returns only a descriptive string. In the concrete example we are using with the exporters, we have a [strong signal in the opentelemetry-specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/sdk.md#export) that indicates concretely that the error types are not actionable by the caller.

If the caller may potentially recover from an error, we will follow [canonical's rust best practices](https://canonical.github.io/rust-best-practices/error-and-panic-discipline.html) and instead preserve the nested error.
