use std::collections::HashMap;
use testcontainers::core::WaitFor;
use testcontainers::Image;

pub struct Collector {
    volumes: HashMap<String, String>,
}

impl Image for Collector {
    type Args = ();

    fn name(&self) -> String {
        "otel/opentelemetry-collector".to_string()
    }

    fn tag(&self) -> String {
        "latest".to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::Nothing]
    }

    fn volumes(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.volumes.iter())
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![
            // 4317,  // gRPC port, defined in Dockerfile
            // 4318,  // HTTP port, defined in Dockerfile
            // 55681, // OpenTelemetry protocol port, defined in Dockerfile
            8888,  // Prometheus metrics exposed by the collector
        ]
    }
}

impl Default for Collector {
    fn default() -> Self {
        Collector {
            volumes: HashMap::from([
                (
                    "./otel-collector-config.yaml".into(),
                    "/etc/otelcol/config.yaml".into(),
                ),
            ]),
        }
    }
}

impl Collector {
    pub fn with_volume(mut self, src: &str, dst: &str) -> Self {
        self.volumes.insert(src.into(), dst.into());
        self
    }
}
