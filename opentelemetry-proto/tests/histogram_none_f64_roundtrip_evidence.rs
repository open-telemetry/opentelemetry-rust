// Evidence repro for PR #3496 review (not for upstream merge).
//
// A HistogramDataPoint with `min`/`max`/`sum` unset (None) serializes to JSON
// containing `"min": null` (etc.), because those Option<f64> fields have no
// `skip_serializing_if`. PR #3496 routes them through
// `serialize_option_f64_special` / `deserialize_option_f64_special`. The
// serializer emits `null` for None, but the deserializer's visitor only
// implements visit_f64/u64/i64/str (no visit_unit/visit_none), so JSON `null`
// fails to deserialize. The library cannot read back its own output.
//
// On `main` (default Option<f64> serde) the same value round-trips fine.
#[cfg(all(feature = "with-serde", feature = "gen-tonic-messages", feature = "metrics"))]
#[test]
fn histogram_data_point_with_none_min_max_sum_round_trips() {
    use opentelemetry_proto::tonic::metrics::v1::HistogramDataPoint;

    let dp = HistogramDataPoint {
        count: 0,
        sum: None,
        min: None,
        max: None,
        ..Default::default()
    };

    let json = serde_json::to_string(&dp).expect("serialize should succeed");
    println!("serialized: {json}");

    let back: HistogramDataPoint =
        serde_json::from_str(&json).expect("deserializing the library's own output should succeed");

    assert_eq!(back, dp);
}
