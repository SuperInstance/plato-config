use crate::config::Config;

pub struct ConfigProfile;

impl ConfigProfile {
    pub fn apply(config: &Config, name: &str) -> Config {
        config.profile(name)
    }

    pub fn available() -> Vec<&'static str> {
        vec!["dev", "staging", "prod", "edge"]
    }
}
