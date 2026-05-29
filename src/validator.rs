use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

pub struct ConfigValidator {
    pub strict: bool,
}

impl ConfigValidator {
    pub fn new() -> Self {
        ConfigValidator { strict: false }
    }

    pub fn validate(&self, config: &Config) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if config.agent_name.is_empty() {
            errors.push(ValidationError {
                path: "agent_name".into(),
                message: "agent_name is required".into(),
            });
        }

        if config.model.is_empty() {
            errors.push(ValidationError {
                path: "model".into(),
                message: "model is required".into(),
            });
        }

        if config.fleet_enabled && config.fleet_bootstrap.is_none() {
            errors.push(ValidationError {
                path: "fleet_bootstrap".into(),
                message: "fleet_bootstrap is required when fleet_enabled is true".into(),
            });
        }

        if config.policy_strict && !config.policy_enabled {
            errors.push(ValidationError {
                path: "policy_strict".into(),
                message: "policy_strict requires policy_enabled".into(),
            });
        }

        let valid_log_levels = ["debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&config.log_level.as_str()) {
            errors.push(ValidationError {
                path: "log_level".into(),
                message: format!("invalid log_level '{}', must be one of {:?}", config.log_level, valid_log_levels),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
