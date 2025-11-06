use opentelemetry_declarative_config::Configurator;

#[tokio::test]
async fn test_configure_telemetry_from_yaml_file_sample1() -> Result<(), Box<dyn std::error::Error>>
{
    let configurator = Configurator::new();
    let result = configurator.configure_telemetry_from_yaml_file("tests/sample1.yaml");
    if let Err(ref e) = result {
        panic!("Failed to configure telemetry from YAML file: {}", e);
    }
    assert!(result.is_ok());
    let telemetry_providers = result.unwrap();
    assert!(telemetry_providers.meter_provider().is_some());
    assert!(telemetry_providers.logs_provider().is_some());
    assert!(telemetry_providers.traces_provider().is_none());

    telemetry_providers.shutdown()?;
    Ok(())
}

#[tokio::test]
async fn test_configure_telemetry_from_yaml_file_with_extra_field(
) -> Result<(), Box<dyn std::error::Error>> {
    let configurator = Configurator::new();
    let result = configurator.configure_telemetry_from_yaml_file("tests/extra_field.yaml");
    assert!(result.is_err());
    if let Err(ref e) = result {
        assert!(e.to_string().contains("unknown field"));
    }
    Ok(())
}

#[tokio::test]
async fn test_configure_telemetry_from_yaml_file_non_implemented_exporter(
) -> Result<(), Box<dyn std::error::Error>> {
    let configurator = Configurator::new();
    let result =
        configurator.configure_telemetry_from_yaml_file("tests/non_maintained_exporter.yaml");
    assert!(result.is_err());
    if let Err(ref e) = result {
        assert!(e.to_string().contains("not maintained"));
    }
    Ok(())
}
