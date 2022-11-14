use std::env;
use std::time::Duration;

/// Default Zipkin collector endpoint
const DEFAULT_COLLECTOR_ENDPOINT: &str = "http://127.0.0.1:9411/api/v2/spans";

/// HTTP endpoint for Zipkin collector.
/// e.g. "http://localhost:9411/api/v2/spans"
const ENV_ENDPOINT: &str = "OTEL_EXPORTER_ZIPKIN_ENDPOINT";

/// Maximum time the Zipkin exporter will wait for each batch export
const ENV_TIMEOUT: &str = "OTEL_EXPORTER_ZIPKIN_TIMEOUT";

/// Default Zipkin timeout in milliseconds
const DEFAULT_COLLECTOR_TIMEOUT: Duration = Duration::from_millis(10_000);

pub(crate) fn get_timeout() -> Duration {
    match env::var(ENV_TIMEOUT).ok().filter(|var| !var.is_empty()) {
        Some(timeout) => match timeout.parse() {
            Ok(timeout) => Duration::from_millis(timeout),
            #[allow(unused)]
            Err(e) => {
                #[cfg(not(feature = "no_stdout"))]
                eprintln!("{} malformed defaulting to 10000: {}", ENV_TIMEOUT, e);
                DEFAULT_COLLECTOR_TIMEOUT
            }
        },
        None => DEFAULT_COLLECTOR_TIMEOUT,
    }
}

pub(crate) fn get_endpoint() -> String {
    match env::var(ENV_ENDPOINT).ok().filter(|var| !var.is_empty()) {
        Some(endpoint) => endpoint,
        None => DEFAULT_COLLECTOR_ENDPOINT.to_string(),
    }
}

#[test]
fn test_collector_defaults() {
    // Ensure the variables are undefined.
    env::remove_var(ENV_TIMEOUT);
    env::remove_var(ENV_ENDPOINT);
    assert_eq!(DEFAULT_COLLECTOR_TIMEOUT, get_timeout());
    assert_eq!(DEFAULT_COLLECTOR_ENDPOINT, get_endpoint());

    // Bad Timeout Value
    env::set_var(ENV_TIMEOUT, "a");
    assert_eq!(DEFAULT_COLLECTOR_TIMEOUT, get_timeout());

    // Good Timeout Value
    env::set_var(ENV_TIMEOUT, "777");
    assert_eq!(Duration::from_millis(777), get_timeout());

    // Custom Endpoint
    let custom_endpoint = "https://example.com/api/v2/spans";
    env::set_var(ENV_ENDPOINT, custom_endpoint);
    assert_eq!(custom_endpoint, get_endpoint());
}
