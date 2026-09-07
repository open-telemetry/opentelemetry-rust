// The OTLP/JSON conformance suite runs once per generated type tree.
#![allow(clippy::duplicate_mod)]

#[cfg(all(feature = "with-serde", feature = "gen-tonic-messages"))]
#[path = ""]
mod tonic {
    use opentelemetry_proto::tonic as proto;
    mod body;
}

#[cfg(feature = "gen-json")]
#[path = ""]
mod json {
    use opentelemetry_proto::json as proto;
    mod body;
}
