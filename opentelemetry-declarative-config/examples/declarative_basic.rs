use opentelemetry_declarative_config::Configurator;

/// Example of configuring OpenTelemetry telemetry using declarative YAML configuration.

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configurator = Configurator::new();
    let config_yaml = r#"
        metrics:
            readers:
                - periodic:
                    exporter:
                        otlp:
                            protocol: http/protobuf
                            endpoint: https://backend:4318
                        stdout:
                            temporality: cumulative
        logs:
            processors:
                - batch:
                    exporter:
                        stdout:
                        otlp:
                            protocol: http/protobuf
                            endpoint: https://backend:4318
        resource:
            service.name: sample-service
            service.version: "1.0.0"
    "#;
    let result = configurator.configure_telemetry_from_yaml(config_yaml.into());
    if let Err(ref e) = result {
        panic!("Failed to configure telemetry from YAML string: {}", e);
    }
    assert!(result.is_ok());
    let telemetry_providers = result.unwrap();
    assert!(telemetry_providers.meter_provider().is_some());
    assert!(telemetry_providers.logs_provider().is_some());
    assert!(telemetry_providers.traces_provider().is_none());

    println!("All the expected telemetry providers were configured successfully. Shutting down...");

    telemetry_providers.shutdown()?;
    Ok(())
}
