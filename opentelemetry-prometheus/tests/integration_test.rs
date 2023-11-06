use std::fs;
use std::path::Path;
use std::time::Duration;

use opentelemetry::metrics::{Meter, MeterProvider as _, Unit};
use opentelemetry::Key;
use opentelemetry::KeyValue;
use opentelemetry_prometheus::ExporterBuilder;
use opentelemetry_sdk::metrics::{new_view, Aggregation, Instrument, SdkMeterProvider, Stream};
use opentelemetry_sdk::resource::{
    EnvResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector,
};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, TELEMETRY_SDK_VERSION};
use prometheus::{Encoder, TextEncoder};

#[test]
fn prometheus_exporter_integration() {
    struct TestCase {
        name: &'static str,
        empty_resource: bool,
        custom_resource_attrs: Vec<KeyValue>,
        #[allow(clippy::type_complexity)]
        record_metrics: Box<dyn Fn(Meter)>,
        builder: ExporterBuilder,
        expected_file: &'static str,
    }

    impl Default for TestCase {
        fn default() -> Self {
            TestCase {
                name: "",
                empty_resource: false,
                custom_resource_attrs: Vec::new(),
                record_metrics: Box::new(|_| {}),
                builder: ExporterBuilder::default(),
                expected_file: "",
            }
        }
    }

    let test_cases = vec![
        TestCase {
            name: "counter",
            expected_file: "counter.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![
                    Key::new("A").string("B"),
                    Key::new("C").string("D"),
                    Key::new("E").bool(true),
                    Key::new("F").i64(42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .with_unit(Unit::new("ms"))
                    .init();
                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
                let attrs2 = vec![
                    Key::new("A").string("D"),
                    Key::new("C").string("B"),
                    Key::new("E").bool(true),
                    Key::new("F").i64(42),
                ];
                counter.add(5.0, &attrs2);
            }),
            ..Default::default()
        },
        TestCase {
            name: "counter with suffixes disabled",
            expected_file: "counter_disabled_suffix.txt",
            builder: ExporterBuilder::default().without_counter_suffixes(),
            record_metrics: Box::new(|meter| {
                let attrs = vec![
                    Key::new("A").string("B"),
                    Key::new("C").string("D"),
                    Key::new("E").bool(true),
                    Key::new("F").i64(42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter without a total suffix")
                    .with_unit(Unit::new("ms"))
                    .init();
                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
                let attrs2 = vec![
                    Key::new("A").string("D"),
                    Key::new("C").string("B"),
                    Key::new("E").bool(true),
                    Key::new("F").i64(42),
                ];
                counter.add(5.0, &attrs2);
            }),
            ..Default::default()
        },
        TestCase {
            name: "gauge",
            expected_file: "gauge.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![Key::new("A").string("B"), Key::new("C").string("D")];
                let gauge = meter
                    .f64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .with_unit(Unit::new("1"))
                    .init();
                gauge.add(1.0, &attrs);
                gauge.add(-0.25, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "histogram",
            expected_file: "histogram.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![Key::new("A").string("B"), Key::new("C").string("D")];
                let histogram = meter
                    .f64_histogram("histogram_baz")
                    .with_description("a very nice histogram")
                    .with_unit(Unit::new("By"))
                    .init();
                histogram.record(23.0, &attrs);
                histogram.record(7.0, &attrs);
                histogram.record(101.0, &attrs);
                histogram.record(105.0, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "sanitized attributes to labels",
            expected_file: "sanitized_labels.txt",
            builder: ExporterBuilder::default().without_units(),
            record_metrics: Box::new(|meter| {
                let attrs = vec![
                    // exact match, value should be overwritten
                    Key::new("A.B").string("X"),
                    Key::new("A.B").string("Q"),
                    // unintended match due to sanitization, values should be concatenated
                    Key::new("C.D").string("Y"),
                    Key::new("C/D").string("Z"),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a sanitary counter")
                    // This unit is not added to
                    .with_unit(Unit::new("By"))
                    .init();
                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "invalid instruments are renamed",
            expected_file: "sanitized_names.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![Key::new("A").string("B"), Key::new("C").string("D")];
                // Valid.
                let mut gauge = meter
                    .f64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .init();
                gauge.add(100., &attrs);
                gauge.add(-25.0, &attrs);

                // Invalid, will be renamed.
                gauge = meter
                    .f64_up_down_counter("invalid.gauge.name")
                    .with_description("a gauge with an invalid name")
                    .init();
                gauge.add(100.0, &attrs);

                let counter = meter
                    .f64_counter("0invalid.counter.name")
                    .with_description("a counter with an invalid name")
                    .init();
                counter.add(100.0, &attrs);

                let histogram = meter
                    .f64_histogram("invalid.hist.name")
                    .with_description("a histogram with an invalid name")
                    .init();
                histogram.record(23.0, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "empty resource",
            empty_resource: true,
            expected_file: "empty_resource.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![
                    Key::new("A").string("B"),
                    Key::new("C").string("D"),
                    Key::new("E").bool(true),
                    Key::new("F").i64(42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .init();
                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "custom resource",
            custom_resource_attrs: vec![Key::new("A").string("B"), Key::new("C").string("D")],
            expected_file: "custom_resource.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![
                    Key::new("A").string("B"),
                    Key::new("C").string("D"),
                    Key::new("E").bool(true),
                    Key::new("F").i64(42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .init();
                counter.add(5., &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "without target_info",
            builder: ExporterBuilder::default().without_target_info(),
            expected_file: "without_target_info.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![
                    Key::new("A").string("B"),
                    Key::new("C").string("D"),
                    Key::new("E").bool(true),
                    Key::new("F").i64(42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .init();
                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "without scope_info",
            builder: ExporterBuilder::default().without_scope_info(),
            expected_file: "without_scope_info.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![Key::new("A").string("B"), Key::new("C").string("D")];
                let gauge = meter
                    .i64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .with_unit(Unit::new("1"))
                    .init();
                gauge.add(2, &attrs);
                gauge.add(-1, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "without scope_info and target_info",
            builder: ExporterBuilder::default()
                .without_scope_info()
                .without_target_info(),
            expected_file: "without_scope_and_target_info.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![Key::new("A").string("B"), Key::new("C").string("D")];
                let counter = meter
                    .u64_counter("bar")
                    .with_description("a fun little counter")
                    .with_unit(Unit::new("By"))
                    .init();
                counter.add(2, &attrs);
                counter.add(1, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "with namespace",
            builder: ExporterBuilder::default().with_namespace("test"),
            expected_file: "with_namespace.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![
                    Key::new("A").string("B"),
                    Key::new("C").string("D"),
                    Key::new("E").bool(true),
                    Key::new("F").i64(42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .init();

                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
            }),
            ..Default::default()
        },
    ];

    for tc in test_cases {
        let registry = prometheus::Registry::new();
        let exporter = tc.builder.with_registry(registry.clone()).build().unwrap();

        let res = if tc.empty_resource {
            Resource::empty()
        } else {
            Resource::from_detectors(
                Duration::from_secs(0),
                vec![
                    Box::new(SdkProvidedResourceDetector),
                    Box::new(EnvResourceDetector::new()),
                    Box::new(TelemetryResourceDetector),
                ],
            )
            .merge(&mut Resource::new(
                vec![
                    // always specify service.name because the default depends on the running OS
                    SERVICE_NAME.string("prometheus_test"),
                    // Overwrite the semconv.TelemetrySDKVersionKey value so we don't need to update every version
                    TELEMETRY_SDK_VERSION.string("latest"),
                ]
                .into_iter()
                .chain(tc.custom_resource_attrs.into_iter()),
            ))
        };

        let provider = SdkMeterProvider::builder()
            .with_resource(res)
            .with_reader(exporter)
            .with_view(
                new_view(
                    Instrument::new().name("histogram_*"),
                    Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                        boundaries: vec![
                            0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 1000.0,
                        ],
                        record_min_max: true,
                    }),
                )
                .unwrap(),
            )
            .build();
        let meter =
            provider.versioned_meter("testmeter", Some("v0.1.0"), None::<&'static str>, None);
        (tc.record_metrics)(meter);

        let content = fs::read_to_string(Path::new("./tests/data").join(tc.expected_file))
            .expect(tc.expected_file);
        gather_and_compare(registry, content, tc.name);
    }
}

fn gather_and_compare(registry: prometheus::Registry, expected: String, name: &'static str) {
    let mut output = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    encoder.encode(&metric_families, &mut output).unwrap();
    let output_string = String::from_utf8(output).unwrap();

    assert_eq!(output_string, expected, "{name}");
}

#[test]
fn multiple_scopes() {
    let registry = prometheus::Registry::new();
    let exporter = ExporterBuilder::default()
        .with_registry(registry.clone())
        .build()
        .unwrap();

    let resource = Resource::from_detectors(
        Duration::from_secs(0),
        vec![
            Box::new(SdkProvidedResourceDetector),
            Box::new(EnvResourceDetector::new()),
            Box::new(TelemetryResourceDetector),
        ],
    )
    .merge(&mut Resource::new(vec![
        // always specify service.name because the default depends on the running OS
        SERVICE_NAME.string("prometheus_test"),
        // Overwrite the semconv.TelemetrySDKVersionKey value so we don't need to update every version
        TELEMETRY_SDK_VERSION.string("latest"),
    ]));

    let provider = SdkMeterProvider::builder()
        .with_reader(exporter)
        .with_resource(resource)
        .build();

    let foo_counter = provider
        .versioned_meter("meterfoo", Some("v0.1.0"), None::<&'static str>, None)
        .u64_counter("foo")
        .with_unit(Unit::new("ms"))
        .with_description("meter foo counter")
        .init();
    foo_counter.add(100, &[KeyValue::new("type", "foo")]);

    let bar_counter = provider
        .versioned_meter("meterbar", Some("v0.1.0"), None::<&'static str>, None)
        .u64_counter("bar")
        .with_unit(Unit::new("ms"))
        .with_description("meter bar counter")
        .init();
    bar_counter.add(200, &[KeyValue::new("type", "bar")]);

    let content = fs::read_to_string("./tests/data/multi_scopes.txt").unwrap();
    gather_and_compare(registry, content, "multi_scope");
}

#[test]
fn duplicate_metrics() {
    struct TestCase {
        name: &'static str,
        custom_resource_attrs: Vec<KeyValue>,
        #[allow(clippy::type_complexity)]
        record_metrics: Box<dyn Fn(Meter, Meter)>,
        builder: ExporterBuilder,
        expected_files: Vec<&'static str>,
    }

    impl Default for TestCase {
        fn default() -> Self {
            TestCase {
                name: "",
                custom_resource_attrs: Vec::new(),
                record_metrics: Box::new(|_, _| {}),
                builder: ExporterBuilder::default(),
                expected_files: Vec::new(),
            }
        }
    }

    let test_cases = vec![
        TestCase {
            name: "no_conflict_two_counters",
            record_metrics: Box::new(|meter_a, meter_b| {
                let foo_a = meter_a
                    .u64_counter("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter counter foo")
                    .init();

                foo_a.add(100, &[KeyValue::new("A", "B")]);

                let foo_b = meter_b
                    .u64_counter("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter counter foo")
                    .init();

                foo_b.add(100, &[KeyValue::new("A", "B")]);
            }),
            expected_files: vec!["no_conflict_two_counters.txt"],
            ..Default::default()
        },
        TestCase {
            name: "no_conflict_two_updowncounters",
            record_metrics: Box::new(|meter_a, meter_b| {
                let foo_a = meter_a
                    .i64_up_down_counter("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter gauge foo")
                    .init();

                foo_a.add(100, &[KeyValue::new("A", "B")]);

                let foo_b = meter_b
                    .i64_up_down_counter("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter gauge foo")
                    .init();

                foo_b.add(100, &[KeyValue::new("A", "B")]);
            }),
            expected_files: vec!["no_conflict_two_updowncounters.txt"],
            ..Default::default()
        },
        TestCase {
            name: "no_conflict_two_histograms",
            record_metrics: Box::new(|meter_a, meter_b| {
                let foo_a = meter_a
                    .i64_histogram("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter histogram foo")
                    .init();

                foo_a.record(100, &[KeyValue::new("A", "B")]);

                let foo_b = meter_b
                    .i64_histogram("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter histogram foo")
                    .init();

                foo_b.record(100, &[KeyValue::new("A", "B")]);
            }),
            expected_files: vec!["no_conflict_two_histograms.txt"],
            ..Default::default()
        },
        TestCase {
            name: "conflict_help_two_counters",
            record_metrics: Box::new(|meter_a, meter_b| {
                let bar_a = meter_a
                    .u64_counter("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter a bar")
                    .init();

                bar_a.add(100, &[KeyValue::new("type", "bar")]);

                let bar_b = meter_b
                    .u64_counter("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter b bar")
                    .init();

                bar_b.add(100, &[KeyValue::new("type", "bar")]);
            }),
            expected_files: vec![
                "conflict_help_two_counters_1.txt",
                "conflict_help_two_counters_2.txt",
            ],
            ..Default::default()
        },
        TestCase {
            name: "conflict_help_two_updowncounters",
            record_metrics: Box::new(|meter_a, meter_b| {
                let bar_a = meter_a
                    .i64_up_down_counter("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter a bar")
                    .init();

                bar_a.add(100, &[KeyValue::new("type", "bar")]);

                let bar_b = meter_b
                    .i64_up_down_counter("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter b bar")
                    .init();

                bar_b.add(100, &[KeyValue::new("type", "bar")]);
            }),
            expected_files: vec![
                "conflict_help_two_updowncounters_1.txt",
                "conflict_help_two_updowncounters_2.txt",
            ],
            ..Default::default()
        },
        TestCase {
            name: "conflict_help_two_histograms",
            record_metrics: Box::new(|meter_a, meter_b| {
                let bar_a = meter_a
                    .i64_histogram("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter a bar")
                    .init();

                bar_a.record(100, &[KeyValue::new("A", "B")]);

                let bar_b = meter_b
                    .i64_histogram("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter b bar")
                    .init();

                bar_b.record(100, &[KeyValue::new("A", "B")]);
            }),
            expected_files: vec![
                "conflict_help_two_histograms_1.txt",
                "conflict_help_two_histograms_2.txt",
            ],
            ..Default::default()
        },
        TestCase {
            name: "conflict_unit_two_counters",
            record_metrics: Box::new(|meter_a, meter_b| {
                let baz_a = meter_a
                    .u64_counter("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter bar")
                    .init();

                baz_a.add(100, &[KeyValue::new("type", "bar")]);

                let baz_b = meter_b
                    .u64_counter("bar")
                    .with_unit(Unit::new("ms"))
                    .with_description("meter bar")
                    .init();

                baz_b.add(100, &[KeyValue::new("type", "bar")]);
            }),
            builder: ExporterBuilder::default().without_units(),
            expected_files: vec!["conflict_unit_two_counters.txt"],
            ..Default::default()
        },
        TestCase {
            name: "conflict_unit_two_updowncounters",
            record_metrics: Box::new(|meter_a, meter_b| {
                let bar_a = meter_a
                    .i64_up_down_counter("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter gauge bar")
                    .init();

                bar_a.add(100, &[KeyValue::new("type", "bar")]);

                let bar_b = meter_b
                    .i64_up_down_counter("bar")
                    .with_unit(Unit::new("ms"))
                    .with_description("meter gauge bar")
                    .init();

                bar_b.add(100, &[KeyValue::new("type", "bar")]);
            }),
            builder: ExporterBuilder::default().without_units(),
            expected_files: vec!["conflict_unit_two_updowncounters.txt"],
            ..Default::default()
        },
        TestCase {
            name: "conflict_unit_two_histograms",
            record_metrics: Box::new(|meter_a, meter_b| {
                let bar_a = meter_a
                    .i64_histogram("bar")
                    .with_unit(Unit::new("By"))
                    .with_description("meter histogram bar")
                    .init();

                bar_a.record(100, &[KeyValue::new("A", "B")]);

                let bar_b = meter_b
                    .i64_histogram("bar")
                    .with_unit(Unit::new("ms"))
                    .with_description("meter histogram bar")
                    .init();

                bar_b.record(100, &[KeyValue::new("A", "B")]);
            }),
            builder: ExporterBuilder::default().without_units(),
            expected_files: vec!["conflict_unit_two_histograms.txt"],
            ..Default::default()
        },
        TestCase {
            name: "conflict_type_counter_and_updowncounter",
            record_metrics: Box::new(|meter_a, _meter_b| {
                let counter = meter_a
                    .u64_counter("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter foo")
                    .init();

                counter.add(100, &[KeyValue::new("type", "foo")]);

                let gauge = meter_a
                    .i64_up_down_counter("foo_total")
                    .with_unit(Unit::new("By"))
                    .with_description("meter foo")
                    .init();

                gauge.add(200, &[KeyValue::new("type", "foo")]);
            }),
            builder: ExporterBuilder::default().without_units(),
            expected_files: vec![
                "conflict_type_counter_and_updowncounter_1.txt",
                "conflict_type_counter_and_updowncounter_2.txt",
            ],
            ..Default::default()
        },
        TestCase {
            name: "conflict_type_histogram_and_updowncounter",
            record_metrics: Box::new(|meter_a, _meter_b| {
                let foo_a = meter_a
                    .i64_up_down_counter("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter gauge foo")
                    .init();

                foo_a.add(100, &[KeyValue::new("A", "B")]);

                let foo_histogram_a = meter_a
                    .i64_histogram("foo")
                    .with_unit(Unit::new("By"))
                    .with_description("meter histogram foo")
                    .init();

                foo_histogram_a.record(100, &[KeyValue::new("A", "B")]);
            }),
            expected_files: vec![
                "conflict_type_histogram_and_updowncounter_1.txt",
                "conflict_type_histogram_and_updowncounter_2.txt",
            ],
            ..Default::default()
        },
    ];

    for tc in test_cases {
        let registry = prometheus::Registry::new();
        let exporter = tc.builder.with_registry(registry.clone()).build().unwrap();

        let resource = Resource::from_detectors(
            Duration::from_secs(0),
            vec![
                Box::new(SdkProvidedResourceDetector),
                Box::new(EnvResourceDetector::new()),
                Box::new(TelemetryResourceDetector),
            ],
        )
        .merge(&mut Resource::new(
            vec![
                // always specify service.name because the default depends on the running OS
                SERVICE_NAME.string("prometheus_test"),
                // Overwrite the semconv.TelemetrySDKVersionKey value so we don't need to update every version
                TELEMETRY_SDK_VERSION.string("latest"),
            ]
            .into_iter()
            .chain(tc.custom_resource_attrs.into_iter()),
        ));

        let provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(exporter)
            .build();

        let meter_a = provider.versioned_meter("ma", Some("v0.1.0"), None::<&'static str>, None);
        let meter_b = provider.versioned_meter("mb", Some("v0.1.0"), None::<&'static str>, None);

        (tc.record_metrics)(meter_a, meter_b);

        let possible_matches = tc
            .expected_files
            .into_iter()
            .map(|f| fs::read_to_string(Path::new("./tests/data").join(f)).expect(f))
            .collect();
        gather_and_compare_multi(registry, possible_matches, tc.name);
    }
}

fn gather_and_compare_multi(
    registry: prometheus::Registry,
    expected: Vec<String>,
    name: &'static str,
) {
    let mut output = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    encoder.encode(&metric_families, &mut output).unwrap();
    let output_string = String::from_utf8(output).unwrap();

    assert!(
        expected.contains(&output_string),
        "mismatched output in {name}"
    )
}
