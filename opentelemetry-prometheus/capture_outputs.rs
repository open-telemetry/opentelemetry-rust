use std::collections::HashSet;
use std::fs;
use std::path::Path;

use opentelemetry::metrics::{Meter, MeterProvider as _};
use opentelemetry::KeyValue;
use opentelemetry::{InstrumentationScope, Key};
use opentelemetry_prometheus::{ExporterBuilder, PrometheusExporter, ResourceSelector};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, TELEMETRY_SDK_VERSION};

fn create_test_resource(custom_attrs: Vec<KeyValue>, empty: bool) -> Resource {
    if empty {
        Resource::builder_empty().build()
    } else {
        Resource::builder()
            .with_attributes(
                vec![
                    KeyValue::new(SERVICE_NAME, "prometheus_test"),
                    KeyValue::new(TELEMETRY_SDK_VERSION, "latest"),
                ]
                .into_iter()
                .chain(custom_attrs.into_iter()),
            )
            .build()
    }
}

fn create_test_scope(name: &'static str) -> InstrumentationScope {
    InstrumentationScope::builder(name)
        .with_version("v0.1.0")
        .with_attributes(vec![KeyValue::new("k", "v")])
        .build()
}

fn create_test_provider(exporter: &PrometheusExporter, resource: Resource) -> SdkMeterProvider {
    SdkMeterProvider::builder()
        .with_resource(resource)
        .with_reader(exporter.clone())
        .build()
}

const BOUNDARIES: &[f64] = &[
    0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 1000.0,
];

fn main() {
    struct TestCase {
        name: &'static str,
        empty_resource: bool,
        custom_resource_attrs: Vec<KeyValue>,
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
                    KeyValue::new("A.B", "X"),
                    KeyValue::new("A.B", "Q"),
                    KeyValue::new("C.D", "Y"),
                    KeyValue::new("C/D", "Z"),
                ];
                let counter = meter
                    .f64_counter("foo")
                    .with_description("a sanitary counter")
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
                let mut gauge = meter
                    .f64_up_down_counter("bar")
                    .with_description("a fun little gauge")
                    .build();
                gauge.add(100., &attrs);
                gauge.add(-25.0, &attrs);

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

        let output = exporter.export().unwrap();
        
        // Read expected
        let expected_path = Path::new("./tests/data").join(tc.expected_file);
        let expected = fs::read_to_string(&expected_path).unwrap_or_else(|_| String::new());
        
        if output != expected {
            println!("MISMATCH: {}", tc.name);
            println!("Expected file: {}", tc.expected_file);
            println!("---ACTUAL OUTPUT---");
            println!("{}", output);
            println!("---END ACTUAL OUTPUT---\n");
        } else {
            println!("PASS: {}", tc.name);
        }
    }
}
