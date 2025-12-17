use std::collections::HashSet;
use std::fs;
use std::path::Path;

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Meter, MeterProvider as _};
use opentelemetry::{InstrumentationScope, Key};
use opentelemetry_prometheus::{ExporterBuilder, PrometheusExporter, ResourceSelector};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, TELEMETRY_SDK_VERSION};

// Helper function to create a test resource with standard attributes
fn create_test_resource(custom_attrs: Vec<KeyValue>, empty: bool) -> Resource {
    if empty {
        Resource::builder_empty().build()
    } else {
        Resource::builder()
            .with_attributes(
                vec![
                    // always specify service.name because the default depends on the running OS
                    KeyValue::new(SERVICE_NAME, "prometheus_test"),
                    // Overwrite the semconv.TelemetrySDKVersionKey value so we don't need to update every version
                    KeyValue::new(TELEMETRY_SDK_VERSION, "latest"),
                ]
                .into_iter()
                .chain(custom_attrs.into_iter()),
            )
            .build()
    }
}

// Helper function to create a test instrumentation scope
fn create_test_scope(name: &'static str) -> InstrumentationScope {
    InstrumentationScope::builder(name)
        .with_version("v0.1.0")
        .with_attributes(vec![KeyValue::new("k", "v")])
        .build()
}

// Helper function to create a provider with exporter and resource
fn create_test_provider(exporter: &PrometheusExporter, resource: Resource) -> SdkMeterProvider {
    SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(exporter.clone())
        .build()
}

const BOUNDARIES: &[f64] = &[
    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 1000.0,
];

const BYTES_BOUNDARIES: &[f64] = &[
    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0, 7500.0,
    10000.0,
];

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
            name: "counter without scope info",
            expected_file: "counter_no_scope_info.txt",
            builder: ExporterBuilder::default().without_scope_info(),
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
            name: "gauge without scope info",
            expected_file: "gauge_no_scope_info.txt",
            builder: ExporterBuilder::default().without_scope_info(),
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
            name: "histogram without scope info",
            expected_file: "histogram_no_scope_info.txt",
            builder: ExporterBuilder::default().without_scope_info(),
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
        let exporter = tc.builder.build().unwrap();
        let res = create_test_resource(tc.custom_resource_attrs, tc.empty_resource);
        let provider = create_test_provider(&exporter, res);
        let scope = create_test_scope("testmeter");

        let meter = provider.meter_with_scope(scope);

        (tc.record_metrics)(meter);

        let content = fs::read_to_string(Path::new("./tests/data").join(tc.expected_file))
            .expect(tc.expected_file);
        gather_and_compare(&exporter, content, tc.name);
    }
}

fn gather_and_compare(exporter: &PrometheusExporter, expected: String, name: &'static str) {
    let output_string = exporter.export().unwrap();

    let expected = get_platform_specific_string(expected);
    let output_string = get_platform_specific_string(output_string);

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
    let exporter = ExporterBuilder::default().build().unwrap();

    let resource = create_test_resource(Vec::new(), false);
    let provider = create_test_provider(&exporter, resource);
    let scope_foo = create_test_scope("meterfoo");

    let foo_counter = provider
        .meter_with_scope(scope_foo)
        .u64_counter("foo")
        .with_unit("ms")
        .with_description("meter foo counter")
        .build();
    foo_counter.add(100, &[KeyValue::new("type", "foo")]);

    let scope_bar = create_test_scope("meterbar");

    let bar_counter = provider
        .meter_with_scope(scope_bar)
        .u64_counter("bar")
        .with_unit("ms")
        .with_description("meter bar counter")
        .build();
    bar_counter.add(200, &[KeyValue::new("type", "bar")]);

    let content = fs::read_to_string("./tests/data/multi_scopes.txt").unwrap();
    gather_and_compare(&exporter, content, "multi_scope");
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
        let exporter = tc.builder.build().unwrap();
        let resource = create_test_resource(tc.custom_resource_attrs, false);
        let provider = create_test_provider(&exporter, resource);
        let scope_ma = create_test_scope("ma");
        let scope_mb = create_test_scope("mb");

        let meter_a = provider.meter_with_scope(scope_ma);
        let meter_b = provider.meter_with_scope(scope_mb);

        (tc.record_metrics)(meter_a, meter_b);

        let possible_matches = tc
            .expected_files
            .into_iter()
            .map(|f| fs::read_to_string(Path::new("./tests/data").join(f)).expect(f))
            .map(get_platform_specific_string)
            .collect();
        gather_and_compare_multi(&exporter, possible_matches, tc.name);
    }
}

fn gather_and_compare_multi(
    exporter: &PrometheusExporter,
    expected: Vec<String>,
    name: &'static str,
) {
    let output_string = exporter.export().unwrap();
    let output_string = get_platform_specific_string(output_string);

    assert!(
        expected.contains(&output_string),
        "mismatched output in {name}"
    )
}

/// Comprehensive end-to-end test covering all OpenTelemetry specification requirements:
/// - All metric types (Counter, UpDownCounter, Histogram) with full metadata
/// - Scope with name, version, schema_url, and custom attributes
/// - Resource with custom attributes
/// - Verification of scope attribute labels (otel_scope_*)
/// - Verification of resource attributes in target_info
/// - Info metric format per spec
#[test]
fn test_comprehensive_spec_compliance() {
    use opentelemetry::{
        InstrumentationScope, KeyValue,
        metrics::{Counter, Histogram, MeterProvider as _, UpDownCounter},
    };
    use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider};

    // Create resource with custom attributes per spec
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", "spec-compliance-test"),
            KeyValue::new("service.version", "1.0.0"),
            KeyValue::new("service.namespace", "testing"),
            KeyValue::new("deployment.environment", "production"),
            KeyValue::new("custom_resource_attr", "resource_value"),
        ])
        .build();

    let exporter = opentelemetry_prometheus::exporter().build().unwrap();

    let provider = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(exporter.clone())
        .build();

    // Create instrumentation scope with full metadata including schema_url and custom attributes
    let scope = InstrumentationScope::builder("comprehensive-test-scope")
        .with_version("2.0.0")
        .with_schema_url("https://opentelemetry.io/schemas/1.20.0")
        .with_attributes(vec![
            KeyValue::new("scope_environment", "test"),
            KeyValue::new("scope_tier", "integration"),
            KeyValue::new("custom_scope_attr", "scope_value"),
        ])
        .build();

    let meter = provider.meter_with_scope(scope);

    // Create Counter with full metadata
    let counter: Counter<u64> = meter
        .u64_counter("test.counter")
        .with_description("A test counter for spec compliance")
        .with_unit("requests")
        .build();

    // Create UpDownCounter with full metadata
    let updown_counter: UpDownCounter<i64> = meter
        .i64_up_down_counter("test.updown")
        .with_description("A test up-down counter")
        .with_unit("active_connections")
        .build();

    // Create Histogram with full metadata
    let histogram: Histogram<f64> = meter
        .f64_histogram("test.histogram")
        .with_description("A test histogram for latency")
        .with_unit("ms")
        .build();

    // Record measurements with attributes
    counter.add(
        100,
        &[
            KeyValue::new("method", "GET"),
            KeyValue::new("status", "200"),
        ],
    );
    counter.add(
        50,
        &[
            KeyValue::new("method", "POST"),
            KeyValue::new("status", "201"),
        ],
    );

    updown_counter.add(5, &[KeyValue::new("region", "us-west")]);
    updown_counter.add(-2, &[KeyValue::new("region", "us-east")]);

    histogram.record(123.45, &[KeyValue::new("endpoint", "/api/v1")]);
    histogram.record(234.56, &[KeyValue::new("endpoint", "/api/v2")]);

    // Force collection
    let _ = provider.force_flush();

    // Export and verify output
    let output = exporter.export().unwrap();

    // Verify counter with description and _total suffix
    assert!(output.contains("# HELP test_counter_total A test counter for spec compliance"));
    assert!(output.contains("# TYPE test_counter_total counter"));
    assert!(output.contains("test_counter_total{method=\"GET\",status=\"200\""));
    assert!(output.contains("test_counter_total{method=\"POST\",status=\"201\""));

    // Verify up-down counter (becomes gauge, no unit suffix added to gauge names)
    assert!(output.contains("# HELP test_updown A test up-down counter"));
    assert!(output.contains("# TYPE test_updown gauge"));
    assert!(output.contains("test_updown{region=\"us-west\""));
    assert!(output.contains("test_updown{region=\"us-east\""));

    // Verify histogram with unit suffix
    assert!(output.contains("# HELP test_histogram_milliseconds A test histogram for latency"));
    assert!(output.contains("# TYPE test_histogram_milliseconds histogram"));
    assert!(output.contains("test_histogram_milliseconds_bucket{endpoint=\"/api/v1\""));
    assert!(output.contains("test_histogram_milliseconds_sum{endpoint=\"/api/v1\""));
    assert!(output.contains("test_histogram_milliseconds_count{endpoint=\"/api/v1\""));

    // Verify scope labels are present on metrics (per spec)
    assert!(output.contains("otel_scope_name=\"comprehensive-test-scope\""));
    assert!(output.contains("otel_scope_version=\"2.0.0\""));
    assert!(output.contains("otel_scope_schema_url=\"https://opentelemetry.io/schemas/1.20.0\""));

    // Verify custom scope attributes with otel_scope_ prefix (per spec)
    assert!(output.contains("otel_scope_scope_environment=\"test\""));
    assert!(output.contains("otel_scope_scope_tier=\"integration\""));
    assert!(output.contains("otel_scope_custom_scope_attr=\"scope_value\""));

    // Verify otel_scope_info metric (per spec)
    assert!(output.contains("# HELP otel_scope_info Instrumentation Scope metadata"));
    assert!(output.contains("# TYPE otel_scope_info gauge"));
    assert!(output.contains("otel_scope_info{"));

    // Verify target_info metric with resource attributes (per spec)
    assert!(output.contains("# HELP target_info Target metadata"));
    assert!(output.contains("# TYPE target_info gauge"));
    assert!(output.contains("target_info{"));
    assert!(output.contains("service_name=\"spec-compliance-test\""));
    assert!(output.contains("service_version=\"1.0.0\""));
    assert!(output.contains("service_namespace=\"testing\""));
    assert!(output.contains("deployment_environment=\"production\""));
    assert!(output.contains("custom_resource_attr=\"resource_value\""));

    // Verify that target_info appears last (per spec recommendation)
    let target_info_pos = output
        .rfind("target_info{")
        .expect("target_info should be present");
    let last_metric_pos = output
        .rfind("# TYPE")
        .expect("should have type declarations");
    assert!(
        target_info_pos > last_metric_pos,
        "target_info should appear after all other TYPE declarations"
    );

    println!("Full output:\n{}", output);
}
