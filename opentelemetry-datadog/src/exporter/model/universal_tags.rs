/// Universal tags - See: https://docs.datadoghq.com/getting_started/tagging/unified_service_tagging

pub struct UniversalTags {
    pub service: UniversalTagField,
    pub env: UniversalTagField,
    pub version: UniversalTagField,
}

impl UniversalTags {
    pub fn new() -> Self {
        UniversalTags {
            service: UniversalTagField::new(UniversalTagEnum::Service),
            env: UniversalTagField::new(UniversalTagEnum::Env),
            version: UniversalTagField::new(UniversalTagEnum::Version),
        }
    }
    pub fn set_service(&mut self, service: Option<String>) {
        self.service.value = service;
    }
    pub fn set_version(&mut self, version: Option<String>) {
        self.version.value = version;
    }
    pub fn set_env(&mut self, env: Option<String>) {
        self.env.value = env;
    }
    pub fn service(&self) -> Option<String> {
        self.service.value.clone()
    }
    pub fn compute_attribute_size(&self) -> u32 {
        self.service.len() + self.env.len() + self.version.len()
    }
}

pub struct UniversalTagField {
    pub value: Option<String>,
    pub kind: UniversalTagEnum,
}

impl UniversalTagField {
    pub fn new(kind: UniversalTagEnum) -> Self {
        UniversalTagField {
            value: kind.find_universal_tag_value(),
            kind,
        }
    }
    pub fn len(&self) -> u32 {
        if self.value.is_some() {
            return 1;
        }
        0
    }
    pub fn get_tag_name(&self) -> &'static str {
        self.kind.get_tag_name()
    }
}

pub enum UniversalTagEnum {
    Service,
    Version,
    Env,
}

impl UniversalTagEnum {
    fn get_env_variable_name(&self) -> &'static str {
        match self {
            UniversalTagEnum::Service => "DD_SERVICE",
            UniversalTagEnum::Version => "DD_VERSION",
            UniversalTagEnum::Env => "DD_ENV",
        }
    }
    fn get_tag_name(&self) -> &'static str {
        match self {
            UniversalTagEnum::Service => "service",
            UniversalTagEnum::Version => "version",
            UniversalTagEnum::Env => "env",
        }
    }
    fn find_universal_tag_value(&self) -> Option<String> {
        let env_name_to_check = self.get_env_variable_name();
        match std::env::var(env_name_to_check) {
            Ok(tag_value) => Some(tag_value.to_lowercase()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service() {
        std::env::set_var("DD_SERVICE", "test-SERVICE");
        let mut universal_tags = UniversalTags::new();
        std::env::remove_var("DD_SERVICE");
        assert_eq!(
            "test-service",
            universal_tags.service.value.clone().unwrap()
        );
        universal_tags.set_service(Some(String::from("new_service")));
        assert_eq!("new_service", universal_tags.service().unwrap());
    }

    #[test]
    fn test_env() {
        std::env::set_var("DD_ENV", "test-env");
        let mut universal_tags = UniversalTags::new();
        std::env::remove_var("DD_ENV");
        assert_eq!("test-env", universal_tags.env.value.clone().unwrap());
        universal_tags.set_env(Some(String::from("new_env")));
        assert_eq!("new_env", universal_tags.env.value.unwrap());
    }

    #[test]
    fn test_version() {
        std::env::set_var("DD_VERSION", "test-version-1.2.3");
        let mut universal_tags = UniversalTags::new();
        std::env::remove_var("DD_VERSION");
        assert_eq!(
            "test-version-1.2.3",
            universal_tags.version.value.clone().unwrap()
        );
        universal_tags.set_version(Some(String::from("new_version")));
        assert_eq!("new_version", universal_tags.version.value.unwrap());
    }
}
