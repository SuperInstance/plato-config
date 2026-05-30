//! # plato-config
//!
//! Configuration management for PLATO rooms — parse, validate, merge.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Sensor config ────────────────────────────────────────────────────

/// Configuration for a single sensor in a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorConfig {
    pub id: String,
    pub sensor_type: String,
    pub sample_rate_hz: f64,
    pub enabled: bool,
    pub labels: HashMap<String, String>,
}

impl SensorConfig {
    /// Create a new sensor config with defaults.
    pub fn new(id: &str, sensor_type: &str) -> Self {
        Self {
            id: id.to_string(),
            sensor_type: sensor_type.to_string(),
            sample_rate_hz: 1.0,
            enabled: true,
            labels: HashMap::new(),
        }
    }

    /// Validate this sensor config.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("sensor id must not be empty".to_string());
        }
        if self.sensor_type.is_empty() {
            return Err("sensor type must not be empty".to_string());
        }
        if self.sample_rate_hz <= 0.0 {
            return Err(format!("sample rate must be positive, got {}", self.sample_rate_hz));
        }
        Ok(())
    }
}

// ── Signal chain config ──────────────────────────────────────────────

/// Configuration for a signal processing chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalChainConfig {
    pub id: String,
    pub steps: Vec<String>,
    pub input_sensor: String,
    pub output_tile_type: String,
    pub params: HashMap<String, f64>,
}

impl SignalChainConfig {
    /// Create a new signal chain config.
    pub fn new(id: &str, input_sensor: &str) -> Self {
        Self {
            id: id.to_string(),
            steps: Vec::new(),
            input_sensor: input_sensor.to_string(),
            output_tile_type: "SensorReading".to_string(),
            params: HashMap::new(),
        }
    }

    /// Validate this signal chain config.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("signal chain id must not be empty".to_string());
        }
        if self.input_sensor.is_empty() {
            return Err("input sensor must not be empty".to_string());
        }
        if self.steps.is_empty() {
            return Err("signal chain must have at least one step".to_string());
        }
        Ok(())
    }
}

// ── Room config ──────────────────────────────────────────────────────

/// Full configuration for a single PLATO room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomConfig {
    pub id: String,
    pub name: String,
    pub sensors: Vec<SensorConfig>,
    pub signal_chains: Vec<SignalChainConfig>,
    pub metadata: HashMap<String, String>,
}

impl RoomConfig {
    /// Create a new room config.
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            sensors: Vec::new(),
            signal_chains: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Validate the entire room config.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.id.is_empty() {
            errors.push("room id must not be empty".to_string());
        }
        if self.name.is_empty() {
            errors.push("room name must not be empty".to_string());
        }

        for sensor in &self.sensors {
            if let Err(e) = sensor.validate() {
                errors.push(format!("sensor '{}': {}", sensor.id, e));
            }
        }

        // Check for duplicate sensor IDs
        let mut seen = std::collections::HashSet::new();
        for sensor in &self.sensors {
            if !seen.insert(&sensor.id) {
                errors.push(format!("duplicate sensor id: '{}'", sensor.id));
            }
        }

        for chain in &self.signal_chains {
            if let Err(e) = chain.validate() {
                errors.push(format!("signal chain '{}': {}", chain.id, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Parse room config from JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("serialize error: {}", e))
    }
}

// ── Fleet config ─────────────────────────────────────────────────────

/// Configuration for an entire fleet of rooms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetConfig {
    pub id: Uuid,
    pub name: String,
    pub rooms: Vec<RoomConfig>,
    pub global_metadata: HashMap<String, String>,
}

impl FleetConfig {
    /// Create a new fleet config.
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            rooms: Vec::new(),
            global_metadata: HashMap::new(),
        }
    }

    /// Validate the fleet config (all rooms must be valid).
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("fleet name must not be empty".to_string());
        }

        if self.rooms.is_empty() {
            errors.push("fleet must have at least one room".to_string());
        }

        let mut seen = std::collections::HashSet::new();
        for room in &self.rooms {
            if !seen.insert(&room.id) {
                errors.push(format!("duplicate room id: '{}'", room.id));
            }
            if let Err(room_errors) = room.validate() {
                errors.extend(room_errors);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Parse fleet config from JSON.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("serialize error: {}", e))
    }
}

// ── Merge ────────────────────────────────────────────────────────────

/// Merge two room configs: defaults are overlaid with overrides.
/// Override values take precedence where present; defaults fill gaps.
pub fn merge(defaults: &RoomConfig, overrides: &RoomConfig) -> RoomConfig {
    let id = if overrides.id.is_empty() { defaults.id.clone() } else { overrides.id.clone() };
    let name = if overrides.name.is_empty() { defaults.name.clone() } else { overrides.name.clone() };

    // Merge sensor lists: combine by id, override wins
    let mut sensor_map: HashMap<String, SensorConfig> = HashMap::new();
    for s in &defaults.sensors {
        sensor_map.insert(s.id.clone(), s.clone());
    }
    for s in &overrides.sensors {
        sensor_map.insert(s.id.clone(), s.clone());
    }

    let mut chain_map: HashMap<String, SignalChainConfig> = HashMap::new();
    for c in &defaults.signal_chains {
        chain_map.insert(c.id.clone(), c.clone());
    }
    for c in &overrides.signal_chains {
        chain_map.insert(c.id.clone(), c.clone());
    }

    let mut metadata = defaults.metadata.clone();
    for (k, v) in &overrides.metadata {
        metadata.insert(k.clone(), v.clone());
    }

    RoomConfig {
        id,
        name,
        sensors: sensor_map.into_values().collect(),
        signal_chains: chain_map.into_values().collect(),
        metadata,
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── SensorConfig ──

    #[test]
    fn sensor_config_new() {
        let sc = SensorConfig::new("s1", "thermal");
        assert_eq!(sc.id, "s1");
        assert_eq!(sc.sensor_type, "thermal");
        assert_eq!(sc.sample_rate_hz, 1.0);
        assert!(sc.enabled);
    }

    #[test]
    fn sensor_config_validate_ok() {
        let sc = SensorConfig::new("s1", "thermal");
        assert!(sc.validate().is_ok());
    }

    #[test]
    fn sensor_config_validate_empty_id() {
        let sc = SensorConfig::new("", "thermal");
        assert!(sc.validate().is_err());
    }

    #[test]
    fn sensor_config_validate_bad_rate() {
        let mut sc = SensorConfig::new("s1", "thermal");
        sc.sample_rate_hz = -1.0;
        assert!(sc.validate().is_err());
    }

    // ── SignalChainConfig ──

    #[test]
    fn signal_chain_validate_ok() {
        let mut sc = SignalChainConfig::new("chain-1", "s1");
        sc.steps.push("filter".to_string());
        assert!(sc.validate().is_ok());
    }

    #[test]
    fn signal_chain_validate_no_steps() {
        let sc = SignalChainConfig::new("chain-1", "s1");
        assert!(sc.validate().is_err());
    }

    #[test]
    fn signal_chain_validate_empty_id() {
        let mut sc = SignalChainConfig::new("", "s1");
        sc.steps.push("x".to_string());
        assert!(sc.validate().is_err());
    }

    // ── RoomConfig ──

    #[test]
    fn room_config_new() {
        let rc = RoomConfig::new("room-1", "Main Lab");
        assert_eq!(rc.id, "room-1");
        assert_eq!(rc.name, "Main Lab");
    }

    #[test]
    fn room_config_validate_ok() {
        let rc = RoomConfig::new("room-1", "Lab");
        assert!(rc.validate().is_ok());
    }

    #[test]
    fn room_config_validate_empty_id() {
        let rc = RoomConfig::new("", "Lab");
        assert!(rc.validate().is_err());
    }

    #[test]
    fn room_config_validate_duplicate_sensors() {
        let mut rc = RoomConfig::new("room-1", "Lab");
        rc.sensors.push(SensorConfig::new("s1", "thermal"));
        rc.sensors.push(SensorConfig::new("s1", "pressure")); // duplicate id
        let errors = rc.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("duplicate sensor id")));
    }

    #[test]
    fn room_config_from_json() {
        let json = r#"{"id":"r1","name":"Room","sensors":[],"signal_chains":[],"metadata":{}}"#;
        let rc = RoomConfig::from_json(json).unwrap();
        assert_eq!(rc.id, "r1");
    }

    #[test]
    fn room_config_to_json_roundtrip() {
        let rc = RoomConfig::new("r1", "Room");
        let json = rc.to_json().unwrap();
        let back = RoomConfig::from_json(&json).unwrap();
        assert_eq!(rc, back);
    }

    #[test]
    fn room_config_from_json_bad() {
        let result = RoomConfig::from_json("not json");
        assert!(result.is_err());
    }

    // ── FleetConfig ──

    #[test]
    fn fleet_config_new() {
        let fc = FleetConfig::new("fleet-1");
        assert_eq!(fc.name, "fleet-1");
        assert!(!fc.id.is_nil());
    }

    #[test]
    fn fleet_config_validate_empty() {
        let fc = FleetConfig::new("fleet-1");
        assert!(fc.validate().is_err()); // no rooms
    }

    #[test]
    fn fleet_config_validate_ok() {
        let mut fc = FleetConfig::new("fleet-1");
        fc.rooms.push(RoomConfig::new("r1", "Room 1"));
        assert!(fc.validate().is_ok());
    }

    #[test]
    fn fleet_config_validate_duplicate_rooms() {
        let mut fc = FleetConfig::new("fleet-1");
        fc.rooms.push(RoomConfig::new("r1", "Room 1"));
        fc.rooms.push(RoomConfig::new("r1", "Room 2"));
        let errors = fc.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("duplicate room id")));
    }

    #[test]
    fn fleet_config_json_roundtrip() {
        let mut fc = FleetConfig::new("fleet-1");
        fc.rooms.push(RoomConfig::new("r1", "Room"));
        let json = fc.to_json().unwrap();
        let back = FleetConfig::from_json(&json).unwrap();
        assert_eq!(fc.name, back.name);
        assert_eq!(fc.rooms.len(), back.rooms.len());
    }

    // ── Merge ──

    #[test]
    fn merge_basic() {
        let defaults = RoomConfig::new("r1", "Default Name");
        let overrides = RoomConfig::new("r1", "Override Name");
        let merged = merge(&defaults, &overrides);
        assert_eq!(merged.name, "Override Name");
    }

    #[test]
    fn merge_fills_gaps() {
        let defaults = RoomConfig::new("r1", "Default");
        let overrides = RoomConfig::new("", "");
        let merged = merge(&defaults, &overrides);
        assert_eq!(merged.id, "r1");
        assert_eq!(merged.name, "Default");
    }

    #[test]
    fn merge_combines_sensors() {
        let defaults = {
            let mut r = RoomConfig::new("r1", "Room");
            r.sensors.push(SensorConfig::new("s1", "thermal"));
            r
        };
        let overrides = {
            let mut r = RoomConfig::new("r1", "Room");
            r.sensors.push(SensorConfig::new("s2", "pressure"));
            r
        };
        let merged = merge(&defaults, &overrides);
        assert_eq!(merged.sensors.len(), 2);
    }

    #[test]
    fn merge_metadata_overlay() {
        let defaults = {
            let mut r = RoomConfig::new("r1", "Room");
            r.metadata.insert("env".to_string(), "prod".to_string());
            r.metadata.insert("region".to_string(), "us-east".to_string());
            r
        };
        let overrides = {
            let mut r = RoomConfig::new("r1", "Room");
            r.metadata.insert("region".to_string(), "eu-west".to_string());
            r
        };
        let merged = merge(&defaults, &overrides);
        assert_eq!(merged.metadata.get("env").unwrap(), "prod");
        assert_eq!(merged.metadata.get("region").unwrap(), "eu-west");
    }
}
