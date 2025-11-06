use opentelemetry_declarative_config::telemetry_config::TelemetryConfig;

#[test]
fn test_telemetry_config_from_yaml_sample1() {
    let yaml_str = std::fs::read_to_string("tests/sample1.yaml").unwrap();
    let config = TelemetryConfig::from_yaml(&yaml_str).unwrap();

    // Validate resource attributes
    let resource = &config.resource;
    if let Some(service_name) = resource.get("service.name") {
        assert_eq!(service_name, "sample-service");
    } else {
        panic!("service.name not found in resource attributes");
    }

    if let Some(service_version) = resource.get("service.version") {
        assert_eq!(service_version, "1.0.0");
    } else {
        panic!("service.version not found in resource attributes");
    }

    // Validate metrics configuration
    assert!(config.metrics.is_some());

    let metrics_config = config.metrics.unwrap();

    assert_eq!(metrics_config.readers.len(), 1);

    let reader = &metrics_config.readers[0];

    if let Some(periodic_reader) = &reader.periodic {
        let otlp_exporter_config = &periodic_reader.exporter.get("otlp").unwrap();

        assert_eq!(
            otlp_exporter_config
                .get("protocol")
                .unwrap()
                .as_str()
                .unwrap(),
            "http/protobuf"
        );
        assert_eq!(
            otlp_exporter_config
                .get("endpoint")
                .unwrap()
                .as_str()
                .unwrap(),
            "https://backend:4318"
        );
    } else {
        panic!("Expected Periodic reader");
    }

    // validate logs configuration
    assert!(config.logs.is_some());
    let logs_config = config.logs.unwrap();
    assert_eq!(logs_config.processors.len(), 1);
    let processor = &logs_config.processors[0];
    if let Some(batch_processor) = &processor.batch {
        let exporter_config = &batch_processor.exporter;
        assert!(exporter_config.get("otlp").is_some());
    } else {
        panic!("Expected Batch processor");
    }
}

#[test]
fn test_telemetry_config_from_empty_yaml() {
    let yaml_str = r#""#;
    let config = TelemetryConfig::from_yaml(yaml_str).unwrap();

    assert!(config.metrics.is_none());
    assert!(config.resource.is_empty());
}
