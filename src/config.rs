use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Interface {
    Cli,
    Api,
    Embedded,
    A2a,
}

impl Default for Interface {
    fn default() -> Self {
        Interface::Cli
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    InProcess,
    Unix,
    Tcp,
}

impl Default for Transport {
    fn default() -> Self {
        Transport::InProcess
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub agent_name: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub interface: Interface,
    #[serde(default)]
    pub fleet_enabled: bool,
    #[serde(default)]
    pub fleet_bootstrap: Option<String>,
    #[serde(default)]
    pub policy_enabled: bool,
    #[serde(default)]
    pub policy_strict: bool,
    #[serde(default)]
    pub sense_modules: Vec<String>,
    #[serde(default)]
    pub transport: Transport,
    #[serde(default)]
    pub log_level: String,
    #[serde(default)]
    pub log_file: Option<String>,
    #[serde(default)]
    pub workspace_path: String,
    #[serde(default)]
    pub data_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            agent_name: "openconstruct".into(),
            model: "default".into(),
            capabilities: vec![],
            modules: vec![],
            interface: Interface::Cli,
            fleet_enabled: false,
            fleet_bootstrap: None,
            policy_enabled: false,
            policy_strict: false,
            sense_modules: vec![],
            transport: Transport::InProcess,
            log_level: "info".into(),
            log_file: None,
            workspace_path: ".".into(),
            data_path: "./data".into(),
        }
    }
}

impl Config {
    pub fn profile(&self, name: &str) -> Config {
        let mut c = self.clone();
        match name {
            "dev" => {
                c.log_level = "debug".into();
                c.policy_strict = false;
            }
            "staging" => {
                c.log_level = "info".into();
                c.policy_strict = true;
            }
            "prod" => {
                c.log_level = "warn".into();
                c.policy_strict = true;
                c.policy_enabled = true;
            }
            "edge" => {
                c.log_level = "debug".into();
                c.fleet_enabled = true;
            }
            _ => {}
        }
        c
    }
}
