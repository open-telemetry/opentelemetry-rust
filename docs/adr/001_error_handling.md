# Error handling patterns in public API interfaces


## Context and Problem Statement
There is uncertainty around how to model errors in  in the `opentelemetry-rust` public API interfaces - that is, APIs facing the consumers. At the time of writing this is an important issue to resolve as moving beginning to move the signals towards RC and eventually a stable release is an urgent priority. 

The situation is as follows; a concrete example is given, but the issue holds across various public traits, in particular the exporters:

* A given public interface in `opentelemetry-sdk`,such as [trait LogExporter](https://github.com/open-telemetry/opentelemetry-rust/blob/3ec4c186ad22944b208ae7c3d38435e735a85c8e/opentelemetry-sdk/src/logs/export.rs#L115) 
* ... exposes multiple discrete actions with logically disjoint error types (e.g. [export](https://github.com/open-telemetry/opentelemetry-rust/blob/3ec4c186ad22944b208ae7c3d38435e735a85c8e/opentelemetry-sdk/src/logs/export.rs#L133-L136) and [shutdown](https://github.com/open-telemetry/opentelemetry-rust/blob/3ec4c186ad22944b208ae7c3d38435e735a85c8e/opentelemetry-sdk/src/logs/export.rs#L139)  - that is, the class of errors returned for each of these actions are foreseeably very different, as is the callers reaction to them
* ... is implemented by multiple concrete types such as `InMemoryLogExporter`, `OtlpLogExporter`, `StdOutLogExporter` that have different error requirements - for instance, an `OtlpLogExporter` will experience network failures, an `InMemoryLogExporter` will not 
* Potentially has operations on the API that, either in the direct implementation, or in a derived utility that utilises the direct implementation, call _multiple_ API actions and therefore need to return an aggregated log type

Today, we have a situation where a single error type is used per API-trait, and some methods simply swallow their errors. In the example above of `LogExporter`, `shutdown` swallows errors, and `export` returns the `LogError` type, a type that could conceptually be thought of as belonging to the entire trait, not a particular method. For the exporters, the [opentelemetry-specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/sdk.md#export) tells us that they need to indicate success or failure, with a distinction made between 'failed' and 'timed out'. 

There are also similar examples in the builders for providers and exports. 

## Related Work

* #2564 
* #2561 
* #2381

## Considered Options

**Option 1: Continue as is**
Continue the status quo, returning a mix of either nothing or the trait-wide error type. This is inconsistent and limits the caller's ability to handle errors. 

**Option 2: Extend trait-wide error type to all methods on trait**
In this option we keep the existing error type, add it to the remaining methods on the trait, and extend the error type to include errors covering the new error conditions. This will mean that callers will have to know how and when to discard errors from a particular API call based on an understanding of which subset of errors that particular call can make. 

Conversely, it will reduce the number of error types in the code base. 

**Option 3: Introduce an error-type per fallible operation, aggregate these into a single trait-wide error type**

For example, in the above we'd have something like:
```rust
pub trait LogExporter {
        
	fn export(...) -> Result<..., ExportError>;
	fn shutdown(...) -> Result<..., ShutdownError>
}

// Concrete errors for an export operation
pub enum ExportError {
    // The distinction between failure and timed out is part of the OTEL spec
    // we need to meet. 

    ExportFailed,  
    
    ExportTimedOut(Duration),
	
	// Allow impls to box up errors that can't be logically mapped
	// back to one of the APIs errors 
	#[error("Unknown error (should not occur): {source:?}")] 
	Unknown { 
		source: Box<dyn std::error::Error + Send + Sync>, 
	},
}

// Aggregate error type for convenience 
// **Note**: This will be added in response to need, not pre-emptively
#[derive(Debug, thiserror::Error)]
pub enum LogError {
	#[error("Export error: {0}")] 
	InitError(#[from] ExportError),
	
	#[error("Shutdown error: {0}")] 
	ShutdownError(#[from] ShutdownError),
}

// A downcast helper for callers that need to work with impl-specific
// unknown errors concretely
impl ExportError {
    /// Attempt to downcast the inner `source` error to a specific type `T`
    pub fn downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T> {
        if let ExportError::Unknown { source } = self {
            source.downcast_ref::<T>()
        } else {
            None
        }
    }
}
```

## Decision Outcome

Chosen option: **"Option 3: Introduce an error-type per fallible operation, aggregate these into a single trait-wide error type"**

### Consequences

* Good, because callers can handle focussed errors with focussed remediation 
* Good, because implementors of the `pub trait`s can box up custom errors in a fashion that follow's [canonical's error and panic discipline](https://canonical.github.io/rust-best-practices/error-and-panic-discipline.html) guide, by avoiding type erasure of impl-specific errors 
* Good, because the per-trait error type (`LogError` for `LogExporter` above) provides consumers of the trait that hit multiple methods in a single method an error type they can use 
* Bad, because there's more code than a single error type
* Bad, because a caller may need to use `downcast_ref` if they have a known trait impl and want to handle a `Unknown` error

