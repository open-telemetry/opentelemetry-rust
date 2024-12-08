use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

use opentelemetry::metrics::{Meter, MeterProvider as _};
use opentelemetry::KeyValue;
use opentelemetry::{InstrumentationScope, Key};
use opentelemetry_prometheus::{ExporterBuilder, ResourceSelector};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::resource::{
    EnvResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector,
};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, TELEMETRY_SDK_VERSION};
use prometheus::{Encoder, TextEncoder};

const BOUNDARIES: &[f64] = &[
    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 1000.0,
];

const BYTES_BOUNDARIES: &[f64] = &[
    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0, 7500.0,
    10000.0,
];

#[ignore = "https://github.com/open-telemetry/opentelemetry-rust/pull/2224"]
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
                    KeyValue::new("A", "B"),
                    KeyValue::new("C", "D"),
                    KeyValue::new("E", true),
                    KeyValue::new("F", 42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .with_unit("ms")
                    .build();
                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
                let attrs2 = vec![
                    KeyValue::new("A", "D"),
                    KeyValue::new("C", "B"),
                    KeyValue::new("E", true),
                    KeyValue::new("F", 42),
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
                    KeyValue::new("A", "B"),
                    KeyValue::new("C", "D"),
                    KeyValue::new("E", true),
                    KeyValue::new("F", 42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter without a total suffix")
                    .with_unit("ms")
                    .build();
                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
                let attrs2 = vec![
                    KeyValue::new("A", "D"),
                    KeyValue::new("C", "B"),
                    KeyValue::new("E", true),
                    KeyValue::new("F", 42),
                ];
                counter.add(5.0, &attrs2);
            }),
            ..Default::default()
        },
        TestCase {
            name: "gauge",
            expected_file: "gauge.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];
                let gauge = meter
                    .f64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .with_unit("1")
                    .build();
                gauge.add(1.0, &attrs);
                gauge.add(-0.25, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "histogram",
            expected_file: "histogram.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];
                let histogram = meter
                    .f64_histogram("histogram_baz")
                    .with_description("a very nice histogram")
                    .with_unit("By")
                    .with_boundaries(BOUNDARIES.to_vec())
                    .build();
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
                    KeyValue::new("A.B", "X"),
                    KeyValue::new("A.B", "Q"),
                    // unintended match due to sanitization, values should be concatenated
                    KeyValue::new("C.D", "Y"),
                    KeyValue::new("C/D", "Z"),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a sanitary counter")
                    // This unit is not added to
                    .with_unit("By")
                    .build();
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
                let attrs = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];
                // Valid.
                let mut gauge = meter
                    .f64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .build();
                gauge.add(100., &attrs);
                gauge.add(-25.0, &attrs);

                // Invalid, will be renamed.
                gauge = meter
                    .f64_up_down_counter("invalid.gauge.name")
                    .with_description("a gauge with an invalid name")
                    .build();
                gauge.add(100.0, &attrs);

                let counter = meter
                    .f64_counter("0invalid.counter.name")
                    .with_description("a counter with an invalid name")
                    .build();
                counter.add(100.0, &attrs);

                let histogram = meter
                    .f64_histogram("invalid.hist.name")
                    .with_description("a histogram with an invalid name")
                    .with_boundaries(BOUNDARIES.to_vec())
                    .build();
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
                    KeyValue::new("A", "B"),
                    KeyValue::new("C", "D"),
                    KeyValue::new("E", true),
                    KeyValue::new("F", 42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .build();
                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "custom resource",
            custom_resource_attrs: vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")],
            expected_file: "custom_resource.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![
                    KeyValue::new("A", "B"),
                    KeyValue::new("C", "D"),
                    KeyValue::new("E", true),
                    KeyValue::new("F", 42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .build();
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
                    KeyValue::new("A", "B"),
                    KeyValue::new("C", "D"),
                    KeyValue::new("E", true),
                    KeyValue::new("F", 42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .build();
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
                let attrs = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];
                let gauge = meter
                    .i64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .with_unit("1")
                    .build();
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
                let attrs = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];
                let counter = meter
                    .u64_counter("bar")
                    .with_description("a fun little counter")
                    .with_unit("By")
                    .build();
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
                    KeyValue::new("A", "B"),
                    KeyValue::new("C", "D"),
                    KeyValue::new("E", true),
                    KeyValue::new("F", 42),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a simple counter")
                    .build();

                counter.add(5.0, &attrs);
                counter.add(10.3, &attrs);
                counter.add(9.0, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "with resource in every metrics",
            builder: ExporterBuilder::default().with_resource_selector(ResourceSelector::All),
            expected_file: "resource_in_every_metrics.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];
                let gauge = meter
                    .i64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .with_unit("1")
                    .build();
                gauge.add(2, &attrs);
                gauge.add(-1, &attrs);
            }),
            ..Default::default()
        },
        TestCase {
            name: "with select resource in every metrics",
            builder: ExporterBuilder::default()
                .with_resource_selector(HashSet::from([Key::new("service.name")])),
            expected_file: "select_resource_in_every_metrics.txt",
            record_metrics: Box::new(|meter| {
                let attrs = vec![KeyValue::new("A", "B"), KeyValue::new("C", "D")];
                let gauge = meter
                    .i64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .with_unit("1")
                    .build();
                gauge.add(2, &attrs);
                gauge.add(-1, &attrs);
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
                    KeyValue::new(SERVICE_NAME, "prometheus_test"),
                    // Overwrite the semconv.TelemetrySDKVersionKey value so we don't need to update every version
                    KeyValue::new(TELEMETRY_SDK_VERSION, "latest"),
                ]
                .into_iter()
                .chain(tc.custom_resource_attrs.into_iter()),
            ))
        };

        let provider = SdkMeterProvider::builder()
            .with_resource(res)
            .with_reader(exporter)
            .build();

        let scope = InstrumentationScope::builder("testmeter")
            .with_version("v0.1.0")
            .with_schema_url("https://opentelemetry.io/schema/1.0.0")
            .with_attributes(vec![KeyValue::new("k", "v")])
            .build();

        let meter = provider.meter_with_scope(scope);

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

    let expected = get_platform_specific_string(expected);
    let output_string = get_platform_specific_string(String::from_utf8(output).unwrap());

    assert_eq!(output_string, expected, "{name}");
}

///  Returns a String which uses the platform specific new line feed character.
fn get_platform_specific_string(input: String) -> String {
    if cfg!(windows) && !input.ends_with("\r\n") && input.ends_with('\n') {
        return input.replace('\n', "\r\n");
    }
    input
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
        KeyValue::new(SERVICE_NAME, "prometheus_test"),
        // Overwrite the semconv.TelemetrySDKVersionKey value so we don't need to update every version
        KeyValue::new(TELEMETRY_SDK_VERSION, "latest"),
    ]));

    let provider = SdkMeterProvider::builder()
        .with_reader(exporter)
        .with_resource(resource)
        .build();

    let scope_foo = InstrumentationScope::builder("meterfoo")
        .with_version("v0.1.0")
        .with_schema_url("https://opentelemetry.io/schema/1.0.0")
        .with_attributes(vec![KeyValue::new("k", "v")])
        .build();

    let foo_counter = provider
        .meter_with_scope(scope_foo)
        .u64_counter("foo")
        .with_unit("ms")
        .with_description("meter foo counter")
        .build();
    foo_counter.add(100, &[KeyValue::new("type", "foo")]);

    let scope_bar = InstrumentationScope::builder("meterbar")
        .with_version("v0.1.0")
        .with_schema_url("https://opentelemetry.io/schema/1.0.0")
        .with_attributes(vec![KeyValue::new("k", "v")])
        .build();

    let bar_counter = provider
        .meter_with_scope(scope_bar)
        .u64_counter("bar")
        .with_unit("ms")
        .with_description("meter bar counter")
        .build();
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
                    .with_unit("By")
                    .with_description("meter counter foo")
                    .build();

                foo_a.add(100, &[KeyValue::new("A", "B")]);

                let foo_b = meter_b
                    .u64_counter("foo")
                    .with_unit("By")
                    .with_description("meter counter foo")
                    .build();

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
                    .with_unit("By")
                    .with_description("meter gauge foo")
                    .build();

                foo_a.add(100, &[KeyValue::new("A", "B")]);

                let foo_b = meter_b
                    .i64_up_down_counter("foo")
                    .with_unit("By")
                    .with_description("meter gauge foo")
                    .build();

                foo_b.add(100, &[KeyValue::new("A", "B")]);
            }),
            expected_files: vec!["no_conflict_two_updowncounters.txt"],
            ..Default::default()
        },
        TestCase {
            name: "no_conflict_two_histograms",
            record_metrics: Box::new(|meter_a, meter_b| {
                let foo_a = meter_a
                    .u64_histogram("foo")
                    .with_unit("By")
                    .with_description("meter histogram foo")
                    .with_boundaries(BYTES_BOUNDARIES.to_vec())
                    .build();

                foo_a.record(100, &[KeyValue::new("A", "B")]);

                let foo_b = meter_b
                    .u64_histogram("foo")
                    .with_unit("By")
                    .with_description("meter histogram foo")
                    .with_boundaries(BYTES_BOUNDARIES.to_vec())
                    .build();

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
                    .with_unit("By")
                    .with_description("meter a bar")
                    .build();

                bar_a.add(100, &[KeyValue::new("type", "bar")]);

                let bar_b = meter_b
                    .u64_counter("bar")
                    .with_unit("By")
                    .with_description("meter b bar")
                    .build();

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
                    .with_unit("By")
                    .with_description("meter a bar")
                    .build();

                bar_a.add(100, &[KeyValue::new("type", "bar")]);

                let bar_b = meter_b
                    .i64_up_down_counter("bar")
                    .with_unit("By")
                    .with_description("meter b bar")
                    .build();

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
                    .u64_histogram("bar")
                    .with_unit("By")
                    .with_description("meter a bar")
                    .with_boundaries(BYTES_BOUNDARIES.to_vec())
                    .build();

                bar_a.record(100, &[KeyValue::new("A", "B")]);

                let bar_b = meter_b
                    .u64_histogram("bar")
                    .with_unit("By")
                    .with_description("meter b bar")
                    .with_boundaries(BYTES_BOUNDARIES.to_vec())
                    .build();

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
                    .with_unit("By")
                    .with_description("meter bar")
                    .build();

                baz_a.add(100, &[KeyValue::new("type", "bar")]);

                let baz_b = meter_b
                    .u64_counter("bar")
                    .with_unit("ms")
                    .with_description("meter bar")
                    .build();

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
                    .with_unit("By")
                    .with_description("meter gauge bar")
                    .build();

                bar_a.add(100, &[KeyValue::new("type", "bar")]);

                let bar_b = meter_b
                    .i64_up_down_counter("bar")
                    .with_unit("ms")
                    .with_description("meter gauge bar")
                    .build();

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
                    .u64_histogram("bar")
                    .with_unit("By")
                    .with_description("meter histogram bar")
                    .with_boundaries(BYTES_BOUNDARIES.to_vec())
                    .build();

                bar_a.record(100, &[KeyValue::new("A", "B")]);

                let bar_b = meter_b
                    .u64_histogram("bar")
                    .with_unit("ms")
                    .with_description("meter histogram bar")
                    .with_boundaries(BYTES_BOUNDARIES.to_vec())
                    .build();

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
                    .with_unit("By")
                    .with_description("meter foo")
                    .build();

                counter.add(100, &[KeyValue::new("type", "foo")]);

                let gauge = meter_a
                    .i64_up_down_counter("foo_total")
                    .with_unit("By")
                    .with_description("meter foo")
                    .build();

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
                    .with_unit("By")
                    .with_description("meter gauge foo")
                    .build();

                foo_a.add(100, &[KeyValue::new("A", "B")]);

                let foo_histogram_a = meter_a
                    .u64_histogram("foo")
                    .with_unit("By")
                    .with_description("meter histogram foo")
                    .with_boundaries(BOUNDARIES.to_vec())
                    .build();

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
                KeyValue::new(SERVICE_NAME, "prometheus_test"),
                // Overwrite the semconv.TelemetrySDKVersionKey value so we don't need to update every version
                KeyValue::new(TELEMETRY_SDK_VERSION, "latest"),
            ]
            .into_iter()
            .chain(tc.custom_resource_attrs.into_iter()),
        ));

        let provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(exporter)
            .build();

        let scope_ma = InstrumentationScope::builder("ma")
            .with_version("v0.1.0")
            .with_schema_url("https://opentelemetry.io/schema/1.0.0")
            .with_attributes(vec![KeyValue::new("k", "v")])
            .build();

        let scope_mb = InstrumentationScope::builder("mb")
            .with_version("v0.1.0")
            .with_schema_url("https://opentelemetry.io/schema/1.0.0")
            .with_attributes(vec![KeyValue::new("k", "v")])
            .build();

        let meter_a = provider.meter_with_scope(scope_ma);
        let meter_b = provider.meter_with_scope(scope_mb);

        (tc.record_metrics)(meter_a, meter_b);

        let possible_matches = tc
            .expected_files
            .into_iter()
            .map(|f| fs::read_to_string(Path::new("./tests/data").join(f)).expect(f))
            .map(get_platform_specific_string)
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

    let output_string = get_platform_specific_string(String::from_utf8(output).unwrap());

    assert!(
        expected.contains(&output_string),
        "mismatched output in {name}"
    )
}
