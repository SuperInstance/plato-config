use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigDiff {
    pub path: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

pub fn diff_configs(old: &Config, new: &Config) -> Vec<ConfigDiff> {
    let mut diffs = Vec::new();

    macro_rules! diff_field {
        ($path:expr, $old_val:expr, $new_val:expr) => {
            let old_s = $old_val.to_string();
            let new_s = $new_val.to_string();
            if old_s != new_s {
                diffs.push(ConfigDiff { path: $path.into(), old_value: Some(old_s), new_value: Some(new_s) });
            }
        };
    }

    macro_rules! diff_vec {
        ($path:expr, $old_val:expr, $new_val:expr) => {
            if $old_val != $new_val {
                diffs.push(ConfigDiff {
                    path: $path.into(),
                    old_value: if $old_val.is_empty() { None } else { Some(format!("{:?}", $old_val)) },
                    new_value: if $new_val.is_empty() { None } else { Some(format!("{:?}", $new_val)) },
                });
            }
        };
    }

    macro_rules! diff_opt {
        ($path:expr, $old_val:expr, $new_val:expr) => {
            let old_s = $old_val.as_ref().map(|v| v.clone());
            let new_s = $new_val.as_ref().map(|v| v.clone());
            if old_s != new_s {
                diffs.push(ConfigDiff { path: $path.into(), old_value: old_s, new_value: new_s });
            }
        };
    }

    diff_field!("agent_name", old.agent_name, new.agent_name);
    diff_field!("model", old.model, new.model);
    diff_vec!("capabilities", old.capabilities, new.capabilities);
    diff_vec!("modules", old.modules, new.modules);
    diff_field!("interface", format!("{:?}", old.interface), format!("{:?}", new.interface));
    diff_field!("fleet_enabled", old.fleet_enabled, new.fleet_enabled);
    diff_opt!("fleet_bootstrap", old.fleet_bootstrap, new.fleet_bootstrap);
    diff_field!("policy_enabled", old.policy_enabled, new.policy_enabled);
    diff_field!("policy_strict", old.policy_strict, new.policy_strict);
    diff_vec!("sense_modules", old.sense_modules, new.sense_modules);
    diff_field!("transport", format!("{:?}", old.transport), format!("{:?}", new.transport));
    diff_field!("log_level", old.log_level, new.log_level);
    diff_opt!("log_file", old.log_file, new.log_file);
    diff_field!("workspace_path", old.workspace_path, new.workspace_path);
    diff_field!("data_path", old.data_path, new.data_path);

    diffs
}
