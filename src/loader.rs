use crate::config::Config;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Toml,
    Json,
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn from_file(path: &str) -> Result<Config, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;
        let fmt = if path.ends_with(".json") {
            Format::Json
        } else {
            Format::Toml
        };
        Self::from_str(&content, fmt)
    }

    pub fn from_str(content: &str, format: Format) -> Result<Config, String> {
        let base = Config::default();
        let overlay = match format {
            Format::Toml => toml::from_str(content).map_err(|e| format!("TOML parse error: {}", e))?,
            Format::Json => serde_json::from_str(content).map_err(|e| format!("JSON parse error: {}", e))?,
        };
        Ok(Self::merge(base, overlay))
    }

    pub fn from_env() -> Config {
        let mut c = Config::default();
        if let Ok(v) = std::env::var("PLATO_AGENT_NAME") {
            c.agent_name = v;
        }
        if let Ok(v) = std::env::var("PLATO_MODEL") {
            c.model = v;
        }
        if let Ok(v) = std::env::var("PLATO_CAPABILITIES") {
            c.capabilities = v.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(v) = std::env::var("PLATO_MODULES") {
            c.modules = v.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(v) = std::env::var("PLATO_INTERFACE") {
            c.interface = match v.as_str() {
                "api" => crate::config::Interface::Api,
                "embedded" => crate::config::Interface::Embedded,
                "a2a" => crate::config::Interface::A2a,
                _ => crate::config::Interface::Cli,
            };
        }
        if let Ok(v) = std::env::var("PLATO_LOG_LEVEL") {
            c.log_level = v;
        }
        if let Ok(v) = std::env::var("PLATO_FLEET_ENABLED") {
            c.fleet_enabled = v == "true" || v == "1";
        }
        if let Ok(v) = std::env::var("PLATO_TRANSPORT") {
            c.transport = match v.as_str() {
                "unix" => crate::config::Transport::Unix,
                "tcp" => crate::config::Transport::Tcp,
                _ => crate::config::Transport::InProcess,
            };
        }
        if let Ok(v) = std::env::var("PLATO_WORKSPACE_PATH") {
            c.workspace_path = v;
        }
        c
    }

    pub fn merge(base: Config, overlay: Config) -> Config {
        let default = Config::default();
        Config {
            agent_name: if overlay.agent_name != default.agent_name { overlay.agent_name } else { base.agent_name },
            model: if overlay.model != default.model { overlay.model } else { base.model },
            capabilities: if !overlay.capabilities.is_empty() { overlay.capabilities } else { base.capabilities },
            modules: if !overlay.modules.is_empty() { overlay.modules } else { base.modules },
            interface: if overlay.interface != default.interface { overlay.interface } else { base.interface },
            fleet_enabled: overlay.fleet_enabled || base.fleet_enabled,
            fleet_bootstrap: overlay.fleet_bootstrap.or(base.fleet_bootstrap),
            policy_enabled: overlay.policy_enabled || base.policy_enabled,
            policy_strict: overlay.policy_strict || base.policy_strict,
            sense_modules: if !overlay.sense_modules.is_empty() { overlay.sense_modules } else { base.sense_modules },
            transport: if overlay.transport != default.transport { overlay.transport } else { base.transport },
            log_level: if overlay.log_level != default.log_level { overlay.log_level } else { base.log_level },
            log_file: overlay.log_file.or(base.log_file),
            workspace_path: if overlay.workspace_path != default.workspace_path { overlay.workspace_path } else { base.workspace_path },
            data_path: if overlay.data_path != default.data_path { overlay.data_path } else { base.data_path },
        }
    }
}
