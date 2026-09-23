//! Error classification for OTLP exporters with protocol-specific throttling support.
//!
//! This module provides error classification functions for HTTP and gRPC protocols,
//! supporting server-provided throttling hints like HTTP Retry-After headers and
//! gRPC RetryInfo metadata.

use crate::retry::RetryErrorType;

#[cfg(feature = "grpc-tonic")]
use tonic_types::StatusExt;

/// HTTP-specific error classification with Retry-After header support.
#[cfg(any(feature = "http-proto", feature = "http-json"))]
pub(crate) mod http {
    use super::*;
    use std::time::Duration;

    /// Classifies HTTP errors based on status code and headers.
    ///
    /// # Arguments
    /// * `status_code` - HTTP status code
    /// * `retry_after_header` - Value of the Retry-After header, if present
    ///
    /// # Retry-After Header Formats
    /// * Seconds: "120"
    /// * HTTP Date: "Fri, 31 Dec 1999 23:59:59 GMT"
    pub(crate) fn classify_http_error(
        status_code: u16,
        retry_after_header: Option<&str>,
    ) -> RetryErrorType {
        match status_code {
            // OTLP/HTTP throttling responses may provide an explicit retry delay.
            429 | 503 => {
                if let Some(retry_after) = retry_after_header {
                    if let Some(duration) = parse_retry_after(retry_after) {
                        return RetryErrorType::Throttled(duration);
                    }
                }
                RetryErrorType::Retryable
            }
            // Other retryable response codes defined by OTLP/HTTP.
            502 | 504 => RetryErrorType::Retryable,
            // The HTTP exporter uses status 0 for failures without a response.
            0 => RetryErrorType::Retryable,
            // All other HTTP response status codes must not be retried.
            _ => RetryErrorType::NonRetryable,
        }
    }

    /// Parses the Retry-After header value.
    ///
    /// Supports both formats:
    /// - Delay seconds: "120"
    /// - HTTP date: "Fri, 31 Dec 1999 23:59:59 GMT"
    ///
    /// Returns None if parsing fails or delay is unreasonable.
    fn parse_retry_after(retry_after: &str) -> Option<Duration> {
        // Try parsing as seconds first
        if let Ok(seconds) = retry_after.trim().parse::<u64>() {
            // Cap at 10 minutes. TODO - what's sensible here?
            let capped_seconds = seconds.min(600);
            return Some(Duration::from_secs(capped_seconds));
        }

        // Try parsing as HTTP date (requires httpdate crate)
        #[cfg(feature = "httpdate")]
        if let Ok(delay_seconds) = parse_http_date_to_delay(retry_after) {
            // Cap at 10 minutes. TODO - what's sensible here?
            let capped_seconds = delay_seconds.min(600);
            return Some(Duration::from_secs(capped_seconds));
        }

        None
    }

    /// Parses HTTP date format and returns delay in seconds from now.
    #[cfg(feature = "httpdate")]
    fn parse_http_date_to_delay(date_str: &str) -> Result<u64, ()> {
        use std::time::SystemTime;

        // Try parse the date; if we fail, propagate an () error up to the caller.
        let target_time = httpdate::parse_http_date(date_str).map_err(|_| ())?;

        let now = SystemTime::now();
        let delay = target_time
            .duration_since(now)
            .unwrap_or(std::time::Duration::ZERO);
        Ok(delay.as_secs())
    }
}

/// gRPC-specific error classification with RetryInfo support.
#[cfg(feature = "grpc-tonic")]
pub(crate) mod grpc {
    use super::*;

    /// Classifies a tonic::Status error
    #[cfg(feature = "grpc-tonic")]
    pub(crate) fn classify_tonic_status(status: &tonic::Status) -> RetryErrorType {
        // Use tonic-types to extract RetryInfo - this is the proper way!
        let retry_delay = status
            .get_details_retry_info()
            .and_then(|retry_info| retry_info.retry_delay);

        classify_grpc_error(status.code(), retry_delay)
    }

    /// Classifies gRPC errors based on status code and metadata.
    ///
    /// Implements the OpenTelemetry OTLP specification for error handling:
    /// https://opentelemetry.io/docs/specs/otlp/
    /// https://github.com/open-telemetry/opentelemetry-proto/blob/main/docs/specification.md#failures
    ///
    /// # Arguments
    /// * `grpc_code` - gRPC status code as tonic::Code enum
    /// * `retry_delay` - Parsed retry delay from RetryInfo metadata, if present
    fn classify_grpc_error(
        grpc_code: tonic::Code,
        retry_delay: Option<std::time::Duration>,
    ) -> RetryErrorType {
        match grpc_code {
            // RESOURCE_EXHAUSTED: Special case per OTLP spec
            // Retryable only if server provides RetryInfo indicating recovery is possible
            tonic::Code::ResourceExhausted => {
                if let Some(delay) = retry_delay {
                    // Server signals recovery is possible - use throttled retry
                    // Cap at 10 minutes. TODO - what's sensible here?
                    return RetryErrorType::Throttled(
                        delay.min(std::time::Duration::from_secs(600)),
                    );
                }
                // No RetryInfo - treat as non-retryable per OTLP spec
                RetryErrorType::NonRetryable
            }

            // UNAVAILABLE may include RetryInfo to signal an explicit throttle delay.
            tonic::Code::Unavailable => match retry_delay.filter(|delay| !delay.is_zero()) {
                Some(delay) => {
                    RetryErrorType::Throttled(delay.min(std::time::Duration::from_secs(600)))
                }
                None => RetryErrorType::Retryable,
            },

            // Retryable errors per OTLP specification
            tonic::Code::Cancelled
            | tonic::Code::DeadlineExceeded
            | tonic::Code::Aborted
            | tonic::Code::OutOfRange
            | tonic::Code::DataLoss => RetryErrorType::Retryable,

            // Non-retryable errors per OTLP specification
            tonic::Code::Unknown
            | tonic::Code::InvalidArgument
            | tonic::Code::NotFound
            | tonic::Code::AlreadyExists
            | tonic::Code::PermissionDenied
            | tonic::Code::FailedPrecondition
            | tonic::Code::Unimplemented
            | tonic::Code::Internal
            | tonic::Code::Unauthenticated => RetryErrorType::NonRetryable,

            // OK should never reach here in error scenarios, but handle gracefully
            tonic::Code::Ok => RetryErrorType::NonRetryable,
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests for HTTP error classification

    #[cfg(any(feature = "http-proto", feature = "http-json"))]
    mod http_tests {
        use crate::retry::RetryErrorType;
        use crate::retry_classification::http::*;
        use std::time::Duration;

        #[test]
        fn test_http_throttling_responses_with_retry_after_seconds() {
            for status_code in [429, 503] {
                let result = classify_http_error(status_code, Some("30"));
                assert_eq!(result, RetryErrorType::Throttled(Duration::from_secs(30)));
            }
        }

        #[test]
        fn test_http_throttling_responses_with_large_retry_after_capped() {
            for status_code in [429, 503] {
                let result = classify_http_error(status_code, Some("900")); // 15 minutes
                assert_eq!(
                    result,
                    RetryErrorType::Throttled(std::time::Duration::from_secs(600))
                ); // Capped at 10 minutes
            }
        }

        #[test]
        fn test_http_throttling_responses_with_invalid_retry_after() {
            for status_code in [429, 503] {
                let result = classify_http_error(status_code, Some("invalid"));
                assert_eq!(result, RetryErrorType::Retryable); // Fallback
            }
        }

        #[test]
        fn test_http_throttling_responses_without_retry_after() {
            for status_code in [429, 503] {
                let result = classify_http_error(status_code, None);
                assert_eq!(result, RetryErrorType::Retryable); // Fallback
            }
        }

        #[test]
        fn test_http_retryable_response_codes() {
            assert_eq!(classify_http_error(502, None), RetryErrorType::Retryable);
            assert_eq!(classify_http_error(503, None), RetryErrorType::Retryable);
            assert_eq!(classify_http_error(504, None), RetryErrorType::Retryable);
        }

        #[test]
        fn test_http_non_retryable_response_codes() {
            for status_code in [400, 401, 403, 404, 408, 499, 500, 501, 505, 599] {
                assert_eq!(
                    classify_http_error(status_code, None),
                    RetryErrorType::NonRetryable
                );
            }
        }

        #[test]
        fn test_http_network_error_is_retryable() {
            assert_eq!(classify_http_error(0, None), RetryErrorType::Retryable);
        }

        #[test]
        #[cfg(feature = "httpdate")]
        fn test_http_throttling_responses_with_retry_after_valid_date() {
            use std::time::SystemTime;

            // Create a time 30 seconds in the future
            let future_time = SystemTime::now() + Duration::from_secs(30);
            let date_str = httpdate::fmt_http_date(future_time);
            for status_code in [429, 503] {
                let result = classify_http_error(status_code, Some(&date_str));
                match result {
                    RetryErrorType::Throttled(duration) => {
                        let secs = duration.as_secs();
                        assert!(
                            (29..=30).contains(&secs),
                            "Expected ~30 seconds, got {}",
                            secs
                        );
                    }
                    _ => panic!("Expected Throttled, got {:?}", result),
                }
            }
        }

        #[test]
        #[cfg(feature = "httpdate")]
        fn test_http_throttling_responses_with_retry_after_invalid_date() {
            for status_code in [429, 503] {
                let result = classify_http_error(status_code, Some("Not a valid date"));
                assert_eq!(result, RetryErrorType::Retryable); // Falls back to retryable
            }
        }

        #[test]
        #[cfg(feature = "httpdate")]
        fn test_http_throttling_responses_with_retry_after_malformed_date() {
            for status_code in [429, 503] {
                let result =
                    classify_http_error(status_code, Some("Sun, 99 Nov 9999 99:99:99 GMT"));
                assert_eq!(result, RetryErrorType::Retryable); // Falls back to retryable
            }
        }
    }

    // Tests for gRPC error classification using public interface
    #[cfg(feature = "grpc-tonic")]
    mod grpc_tests {
        use crate::retry::RetryErrorType;
        use crate::retry_classification::grpc::classify_tonic_status;
        use tonic_types::{ErrorDetails, StatusExt};

        #[test]
        fn test_grpc_resource_exhausted_with_retry_info() {
            let error_details =
                ErrorDetails::with_retry_info(Some(std::time::Duration::from_secs(45)));
            let status = tonic::Status::with_error_details(
                tonic::Code::ResourceExhausted,
                "rate limited",
                error_details,
            );
            let result = classify_tonic_status(&status);
            assert_eq!(
                result,
                RetryErrorType::Throttled(std::time::Duration::from_secs(45))
            );
        }

        #[test]
        fn test_grpc_resource_exhausted_with_large_retry_info_capped() {
            let error_details =
                ErrorDetails::with_retry_info(Some(std::time::Duration::from_secs(900))); // 15 minutes
            let status = tonic::Status::with_error_details(
                tonic::Code::ResourceExhausted,
                "rate limited",
                error_details,
            );
            let result = classify_tonic_status(&status);
            assert_eq!(
                result,
                RetryErrorType::Throttled(std::time::Duration::from_secs(600))
            ); // Capped at 10 minutes
        }

        #[test]
        fn test_grpc_resource_exhausted_without_retry_info() {
            let status = tonic::Status::new(tonic::Code::ResourceExhausted, "rate limited");
            let result = classify_tonic_status(&status);
            // Per OTLP spec: RESOURCE_EXHAUSTED without RetryInfo is non-retryable
            assert_eq!(result, RetryErrorType::NonRetryable);
        }

        #[test]
        fn test_grpc_unavailable_with_retry_info() {
            let error_details =
                ErrorDetails::with_retry_info(Some(std::time::Duration::from_secs(45)));
            let status = tonic::Status::with_error_details(
                tonic::Code::Unavailable,
                "service unavailable",
                error_details,
            );

            assert_eq!(
                classify_tonic_status(&status),
                RetryErrorType::Throttled(std::time::Duration::from_secs(45))
            );
        }

        #[test]
        fn test_grpc_unavailable_with_fractional_retry_info() {
            let error_details =
                ErrorDetails::with_retry_info(Some(std::time::Duration::from_millis(500)));
            let status = tonic::Status::with_error_details(
                tonic::Code::Unavailable,
                "service unavailable",
                error_details,
            );

            assert_eq!(
                classify_tonic_status(&status),
                RetryErrorType::Throttled(std::time::Duration::from_millis(500))
            );
        }

        #[test]
        fn test_grpc_unavailable_with_large_retry_info_capped() {
            let error_details =
                ErrorDetails::with_retry_info(Some(std::time::Duration::from_secs(900)));
            let status = tonic::Status::with_error_details(
                tonic::Code::Unavailable,
                "service unavailable",
                error_details,
            );

            assert_eq!(
                classify_tonic_status(&status),
                RetryErrorType::Throttled(std::time::Duration::from_secs(600))
            );
        }

        #[test]
        fn test_grpc_unavailable_without_positive_retry_info() {
            let without_retry_info =
                tonic::Status::new(tonic::Code::Unavailable, "service unavailable");
            assert_eq!(
                classify_tonic_status(&without_retry_info),
                RetryErrorType::Retryable
            );

            let error_details = ErrorDetails::with_retry_info(Some(std::time::Duration::ZERO));
            let zero_retry_info = tonic::Status::with_error_details(
                tonic::Code::Unavailable,
                "service unavailable",
                error_details,
            );
            assert_eq!(
                classify_tonic_status(&zero_retry_info),
                RetryErrorType::Retryable
            );
        }

        #[test]
        fn test_grpc_retryable_errors() {
            // Test all retryable errors per OTLP specification
            let cancelled = tonic::Status::new(tonic::Code::Cancelled, "cancelled");
            assert_eq!(classify_tonic_status(&cancelled), RetryErrorType::Retryable);

            let deadline_exceeded =
                tonic::Status::new(tonic::Code::DeadlineExceeded, "deadline exceeded");
            assert_eq!(
                classify_tonic_status(&deadline_exceeded),
                RetryErrorType::Retryable
            );

            let aborted = tonic::Status::new(tonic::Code::Aborted, "aborted");
            assert_eq!(classify_tonic_status(&aborted), RetryErrorType::Retryable);

            let out_of_range = tonic::Status::new(tonic::Code::OutOfRange, "out of range");
            assert_eq!(
                classify_tonic_status(&out_of_range),
                RetryErrorType::Retryable
            );

            let unavailable = tonic::Status::new(tonic::Code::Unavailable, "unavailable");
            assert_eq!(
                classify_tonic_status(&unavailable),
                RetryErrorType::Retryable
            );

            let data_loss = tonic::Status::new(tonic::Code::DataLoss, "data loss");
            assert_eq!(classify_tonic_status(&data_loss), RetryErrorType::Retryable);
        }

        #[test]
        fn test_grpc_non_retryable_errors() {
            // Test all non-retryable errors per OTLP specification
            let unknown = tonic::Status::new(tonic::Code::Unknown, "unknown");
            assert_eq!(
                classify_tonic_status(&unknown),
                RetryErrorType::NonRetryable
            );

            let invalid_argument =
                tonic::Status::new(tonic::Code::InvalidArgument, "invalid argument");
            assert_eq!(
                classify_tonic_status(&invalid_argument),
                RetryErrorType::NonRetryable
            );

            let not_found = tonic::Status::new(tonic::Code::NotFound, "not found");
            assert_eq!(
                classify_tonic_status(&not_found),
                RetryErrorType::NonRetryable
            );

            let already_exists = tonic::Status::new(tonic::Code::AlreadyExists, "already exists");
            assert_eq!(
                classify_tonic_status(&already_exists),
                RetryErrorType::NonRetryable
            );

            let permission_denied =
                tonic::Status::new(tonic::Code::PermissionDenied, "permission denied");
            assert_eq!(
                classify_tonic_status(&permission_denied),
                RetryErrorType::NonRetryable
            );

            let failed_precondition =
                tonic::Status::new(tonic::Code::FailedPrecondition, "failed precondition");
            assert_eq!(
                classify_tonic_status(&failed_precondition),
                RetryErrorType::NonRetryable
            );

            let unimplemented = tonic::Status::new(tonic::Code::Unimplemented, "unimplemented");
            assert_eq!(
                classify_tonic_status(&unimplemented),
                RetryErrorType::NonRetryable
            );

            let internal = tonic::Status::new(tonic::Code::Internal, "internal error");
            assert_eq!(
                classify_tonic_status(&internal),
                RetryErrorType::NonRetryable
            );

            let unauthenticated =
                tonic::Status::new(tonic::Code::Unauthenticated, "unauthenticated");
            assert_eq!(
                classify_tonic_status(&unauthenticated),
                RetryErrorType::NonRetryable
            );
        }

        #[test]
        fn test_grpc_ok_code_handled() {
            // OK status should be handled gracefully (though unlikely in error scenarios)
            let ok = tonic::Status::new(tonic::Code::Ok, "success");
            assert_eq!(classify_tonic_status(&ok), RetryErrorType::NonRetryable);
        }

        // Tests for tonic-types RetryInfo integration
        #[cfg(feature = "grpc-tonic")]
        mod retry_info_tests {
            use super::*;
            use crate::retry_classification::grpc::classify_tonic_status;
            use tonic_types::{ErrorDetails, StatusExt};

            #[test]
            fn test_classify_status_with_retry_info() {
                // Create a tonic::Status with RetryInfo using proper StatusExt API
                let error_details =
                    ErrorDetails::with_retry_info(Some(std::time::Duration::from_secs(30)));
                let status = tonic::Status::with_error_details(
                    tonic::Code::ResourceExhausted,
                    "rate limited",
                    error_details,
                );

                // Test classification
                let result = classify_tonic_status(&status);
                assert_eq!(
                    result,
                    RetryErrorType::Throttled(std::time::Duration::from_secs(30))
                );
            }

            #[test]
            fn test_classify_status_with_fractional_retry_info() {
                // Create a tonic::Status with fractional seconds RetryInfo
                let error_details =
                    ErrorDetails::with_retry_info(Some(std::time::Duration::from_millis(500)));
                let status = tonic::Status::with_error_details(
                    tonic::Code::ResourceExhausted,
                    "rate limited",
                    error_details,
                );

                // Fractional seconds are preserved.
                let result = classify_tonic_status(&status);
                assert_eq!(
                    result,
                    RetryErrorType::Throttled(std::time::Duration::from_millis(500))
                );
            }

            #[test]
            fn test_classify_status_without_retry_info() {
                // Status with resource_exhausted but no RetryInfo
                let status = tonic::Status::new(tonic::Code::ResourceExhausted, "rate limited");

                // Per OTLP spec: should be non-retryable without RetryInfo
                let result = classify_tonic_status(&status);
                assert_eq!(result, RetryErrorType::NonRetryable);
            }

            #[test]
            fn test_classify_status_non_retryable_error() {
                // Status with non-retryable error code
                let status = tonic::Status::new(tonic::Code::InvalidArgument, "bad request");

                let result = classify_tonic_status(&status);
                assert_eq!(result, RetryErrorType::NonRetryable);
            }

            #[test]
            fn test_classify_status_retryable_error() {
                // Status with retryable error code
                let status = tonic::Status::new(tonic::Code::Unavailable, "service unavailable");

                let result = classify_tonic_status(&status);
                assert_eq!(result, RetryErrorType::Retryable);
            }

            #[test]
            fn test_classify_status_large_retry_delay() {
                // Test with large retry delay - should be capped at 10 minutes
                let error_details =
                    ErrorDetails::with_retry_info(Some(std::time::Duration::from_secs(3600))); // 1 hour
                let status = tonic::Status::with_error_details(
                    tonic::Code::ResourceExhausted,
                    "rate limited",
                    error_details,
                );

                let result = classify_tonic_status(&status);
                // Should be capped at 10 minutes (600 seconds)
                assert_eq!(
                    result,
                    RetryErrorType::Throttled(std::time::Duration::from_secs(600))
                );
            }

            #[test]
            fn test_status_ext_get_details() {
                // Test that StatusExt works correctly
                let error_details =
                    ErrorDetails::with_retry_info(Some(std::time::Duration::from_secs(45)));
                let status = tonic::Status::with_error_details(
                    tonic::Code::ResourceExhausted,
                    "rate limited",
                    error_details,
                );

                // Direct extraction should work
                let extracted = status.get_details_retry_info();
                assert!(extracted.is_some());

                let retry_delay = extracted.unwrap().retry_delay;
                assert_eq!(retry_delay, Some(std::time::Duration::from_secs(45)));
            }
        }
    }
}
