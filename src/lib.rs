mod config;
mod loader;
mod validator;
mod watcher;
mod diff;
mod profile;

pub use config::{Config, Interface, Transport};
pub use loader::{ConfigLoader, Format};
pub use validator::{ConfigValidator, ValidationError};
pub use watcher::ConfigWatcher;
pub use diff::ConfigDiff;
pub use profile::ConfigProfile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = Config::default();
        assert_eq!(c.agent_name, "openconstruct");
        assert_eq!(c.model, "default");
        assert!(c.capabilities.is_empty());
        assert!(c.modules.is_empty());
        assert_eq!(c.interface, Interface::Cli);
        assert!(!c.fleet_enabled);
        assert!(!c.policy_enabled);
        assert!(c.sense_modules.is_empty());
        assert_eq!(c.transport, Transport::InProcess);
        assert_eq!(c.log_level, "info");
    }

    #[test]
    fn test_load_from_toml() {
        let toml = r#"
            agent_name = "test-agent"
            model = "gpt-4"
            capabilities = ["code", "search"]
            log_level = "debug"
            interface = "api"
            transport = "tcp"
        "#;
        let c = ConfigLoader::from_str(toml, Format::Toml).unwrap();
        assert_eq!(c.agent_name, "test-agent");
        assert_eq!(c.model, "gpt-4");
        assert_eq!(c.capabilities, vec!["code", "search"]);
        assert_eq!(c.interface, Interface::Api);
        assert_eq!(c.transport, Transport::Tcp);
    }

    #[test]
    fn test_load_from_json() {
        let json = r#"{
            "agent_name": "json-agent",
            "model": "claude",
            "capabilities": ["write"],
            "log_level": "warn",
            "interface": "embedded",
            "transport": "unix"
        }"#;
        let c = ConfigLoader::from_str(json, Format::Json).unwrap();
        assert_eq!(c.agent_name, "json-agent");
        assert_eq!(c.interface, Interface::Embedded);
        assert_eq!(c.transport, Transport::Unix);
    }

    #[test]
    fn test_merge_configs() {
        let mut base = Config::default();
        base.agent_name = "base".into();
        base.model = "v1".into();
        base.capabilities = vec!["a".into()];

        let mut overlay = Config::default();
        overlay.agent_name = "overlay".into();
        overlay.capabilities = vec!["b".into(), "c".into()];

        let merged = ConfigLoader::merge(base, overlay);
        assert_eq!(merged.agent_name, "overlay");
        assert_eq!(merged.model, "v1"); // overlay kept default "default", merge takes non-default
        assert_eq!(merged.capabilities, vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn test_validate_valid_config() {
        let mut c = Config::default();
        c.agent_name = "valid".into();
        c.model = "test-model".into();
        let v = ConfigValidator::new();
        assert!(v.validate(&c).is_ok());
    }

    #[test]
    fn test_validate_missing_required() {
        let mut c = Config::default();
        c.agent_name = String::new();
        let v = ConfigValidator::new();
        let errs = v.validate(&c).unwrap_err();
        assert!(errs.iter().any(|e| e.path == "agent_name"));
    }

    #[test]
    fn test_validate_conflicting_settings() {
        let mut c = Config::default();
        c.agent_name = "x".into();
        c.model = "y".into();
        c.fleet_enabled = true;
        c.fleet_bootstrap = None;
        let v = ConfigValidator::new();
        let errs = v.validate(&c).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("fleet_bootstrap")));
    }

    #[test]
    fn test_diff_detects_changes() {
        let a = Config::default();
        let mut b = Config::default();
        b.agent_name = "changed".into();
        let diffs = diff::diff_configs(&a, &b);
        assert!(diffs.iter().any(|d| d.path == "agent_name" && d.old_value.as_deref() == Some("openconstruct") && d.new_value.as_deref() == Some("changed")));
    }

    #[test]
    fn test_diff_detects_additions() {
        let a = Config::default();
        let mut b = Config::default();
        b.capabilities = vec!["new-cap".into()];
        let diffs = diff::diff_configs(&a, &b);
        assert!(diffs.iter().any(|d| d.path == "capabilities" && d.old_value.is_none() && d.new_value.is_some()));
    }

    #[test]
    fn test_diff_detects_removals() {
        let mut a = Config::default();
        a.capabilities = vec!["old-cap".into()];
        let b = Config::default();
        let diffs = diff::diff_configs(&a, &b);
        assert!(diffs.iter().any(|d| d.path == "capabilities" && d.old_value.is_some() && d.new_value.is_none()));
    }

    #[test]
    fn test_profile_overrides() {
        let mut c = Config::default();
        c.model = "base-model".into();
        let profiled = c.profile("prod");
        assert!(profiled.model.contains("prod") || profiled.log_level == "warn");
    }

    #[test]
    fn test_from_env() {
        std::env::set_var("PLATO_AGENT_NAME", "env-agent");
        std::env::set_var("PLATO_MODEL", "env-model");
        let c = ConfigLoader::from_env();
        assert_eq!(c.agent_name, "env-agent");
        assert_eq!(c.model, "env-model");
        std::env::remove_var("PLATO_AGENT_NAME");
        std::env::remove_var("PLATO_MODEL");
    }

    #[test]
    fn test_load_from_file_toml() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("plato_test_file");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "agent_name = \"file-agent\"\nmodel = \"file-model\"\n").unwrap();
        let c = ConfigLoader::from_file(path.to_str().unwrap()).unwrap();
        assert_eq!(c.agent_name, "file-agent");
        std::fs::remove_dir_all(&dir).ok();
    }
}
