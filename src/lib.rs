/// Scoring weight configuration for tile ranking.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoringWeights {
    pub keyword: f64,   // default 0.30
    pub ghost: f64,     // default 0.15
    pub belief: f64,    // default 0.25
    pub domain: f64,    // default 0.20
    pub frequency: f64, // default 0.10
}

impl ScoringWeights {
    pub fn default() -> Self {
        Self {
            keyword: 0.30,
            ghost: 0.15,
            belief: 0.25,
            domain: 0.20,
            frequency: 0.10,
        }
    }
}

/// Global PLATO runtime configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct PlatoConfig {
    /// Minimum score threshold for deadband pass (default: 0.5)
    pub deadband_threshold: f64,
    /// Maximum number of tiles returned per search (default: 20)
    pub search_limit: usize,
    /// Ghost score increment per decay tick (default: 0.04)
    pub decay_rate: f64,
    /// Maximum number of rooms in the fleet (default: 64)
    pub max_rooms: usize,
    /// Tile scoring weights
    pub scoring_weights: ScoringWeights,
}

/// Return the default configuration with all values set to sensible defaults.
pub fn default_config() -> PlatoConfig {
    PlatoConfig {
        deadband_threshold: 0.5,
        search_limit: 20,
        decay_rate: 0.04,
        max_rooms: 64,
        scoring_weights: ScoringWeights::default(),
    }
}

/// Read configuration from environment variables.
/// Reads: PLATO_DEADBAND_THRESHOLD, PLATO_SEARCH_LIMIT, PLATO_DECAY_RATE,
///        PLATO_MAX_ROOMS (all optional, use defaults for missing vars).
/// Parses with .parse::<f64>() / .parse::<usize>() — ignores parse errors (uses default).
pub fn from_env() -> PlatoConfig {
    let defaults = default_config();

    let deadband_threshold = std::env::var("PLATO_DEADBAND_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(defaults.deadband_threshold);

    let search_limit = std::env::var("PLATO_SEARCH_LIMIT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(defaults.search_limit);

    let decay_rate = std::env::var("PLATO_DECAY_RATE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(defaults.decay_rate);

    let max_rooms = std::env::var("PLATO_MAX_ROOMS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(defaults.max_rooms);

    PlatoConfig {
        deadband_threshold,
        search_limit,
        decay_rate,
        max_rooms,
        scoring_weights: defaults.scoring_weights,
    }
}

/// Validate configuration, returning a list of error strings.
/// Rules:
///   - deadband_threshold must be in [0.0, 1.0]
///   - search_limit must be >= 1
///   - decay_rate must be in (0.0, 1.0]
///   - max_rooms must be >= 1
///   - scoring_weights components must all be >= 0.0
///   - scoring_weights components must sum to approximately 1.0 (within 0.01)
pub fn validate(config: &PlatoConfig) -> Vec<String> {
    let mut errors = Vec::new();

    if config.deadband_threshold < 0.0 || config.deadband_threshold > 1.0 {
        errors.push(format!(
            "deadband_threshold must be in [0.0, 1.0], got {}",
            config.deadband_threshold
        ));
    }

    if config.search_limit < 1 {
        errors.push("search_limit must be >= 1".to_string());
    }

    if config.decay_rate <= 0.0 || config.decay_rate > 1.0 {
        errors.push(format!(
            "decay_rate must be in (0.0, 1.0], got {}",
            config.decay_rate
        ));
    }

    if config.max_rooms < 1 {
        errors.push("max_rooms must be >= 1".to_string());
    }

    let w = &config.scoring_weights;
    if w.keyword < 0.0 {
        errors.push(format!("scoring_weights.keyword must be >= 0.0, got {}", w.keyword));
    }
    if w.ghost < 0.0 {
        errors.push(format!("scoring_weights.ghost must be >= 0.0, got {}", w.ghost));
    }
    if w.belief < 0.0 {
        errors.push(format!("scoring_weights.belief must be >= 0.0, got {}", w.belief));
    }
    if w.domain < 0.0 {
        errors.push(format!("scoring_weights.domain must be >= 0.0, got {}", w.domain));
    }
    if w.frequency < 0.0 {
        errors.push(format!("scoring_weights.frequency must be >= 0.0, got {}", w.frequency));
    }

    let sum = w.keyword + w.ghost + w.belief + w.domain + w.frequency;
    if (sum - 1.0).abs() > 0.01 {
        errors.push(format!(
            "scoring_weights must sum to approximately 1.0 (within 0.01), got {}",
            sum
        ));
    }

    errors
}

/// Merge base config with override — override values replace base values when non-zero/non-default.
/// Override logic: if override field differs from default_config() value, use override, else use base.
pub fn merge(base: PlatoConfig, override_cfg: PlatoConfig) -> PlatoConfig {
    let defaults = default_config();

    let deadband_threshold = if override_cfg.deadband_threshold != defaults.deadband_threshold {
        override_cfg.deadband_threshold
    } else {
        base.deadband_threshold
    };

    let search_limit = if override_cfg.search_limit != defaults.search_limit {
        override_cfg.search_limit
    } else {
        base.search_limit
    };

    let decay_rate = if override_cfg.decay_rate != defaults.decay_rate {
        override_cfg.decay_rate
    } else {
        base.decay_rate
    };

    let max_rooms = if override_cfg.max_rooms != defaults.max_rooms {
        override_cfg.max_rooms
    } else {
        base.max_rooms
    };

    let scoring_weights = if override_cfg.scoring_weights != defaults.scoring_weights {
        override_cfg.scoring_weights
    } else {
        base.scoring_weights
    };

    PlatoConfig {
        deadband_threshold,
        search_limit,
        decay_rate,
        max_rooms,
        scoring_weights,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let cfg = default_config();
        assert_eq!(cfg.deadband_threshold, 0.5);
        assert_eq!(cfg.search_limit, 20);
        assert_eq!(cfg.decay_rate, 0.04);
        assert_eq!(cfg.max_rooms, 64);
        assert_eq!(cfg.scoring_weights.keyword, 0.30);
        assert_eq!(cfg.scoring_weights.ghost, 0.15);
        assert_eq!(cfg.scoring_weights.belief, 0.25);
        assert_eq!(cfg.scoring_weights.domain, 0.20);
        assert_eq!(cfg.scoring_weights.frequency, 0.10);
    }

    #[test]
    fn test_validate_valid_config() {
        let cfg = default_config();
        let errors = validate(&cfg);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_validate_deadband_threshold_out_of_range() {
        let mut cfg = default_config();
        cfg.deadband_threshold = 1.5;
        let errors = validate(&cfg);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("deadband_threshold")));

        cfg.deadband_threshold = -0.1;
        let errors = validate(&cfg);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("deadband_threshold")));
    }

    #[test]
    fn test_validate_search_limit_zero() {
        let mut cfg = default_config();
        cfg.search_limit = 0;
        let errors = validate(&cfg);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("search_limit")));
    }

    #[test]
    fn test_validate_decay_rate_zero() {
        let mut cfg = default_config();
        cfg.decay_rate = 0.0;
        let errors = validate(&cfg);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("decay_rate")));
    }

    #[test]
    fn test_validate_weights_not_summing_to_one() {
        let mut cfg = default_config();
        cfg.scoring_weights.keyword = 0.50; // sum becomes 1.20
        let errors = validate(&cfg);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("scoring_weights")));
    }

    #[test]
    fn test_merge_uses_override_when_different() {
        let base = default_config();
        let mut override_cfg = default_config();
        override_cfg.search_limit = 50;
        override_cfg.deadband_threshold = 0.8;

        let merged = merge(base, override_cfg);
        assert_eq!(merged.search_limit, 50);
        assert_eq!(merged.deadband_threshold, 0.8);
    }

    #[test]
    fn test_merge_preserves_base_when_override_matches_default() {
        let mut base = default_config();
        base.search_limit = 100;
        base.deadband_threshold = 0.7;
        let override_cfg = default_config(); // all values match defaults

        let merged = merge(base, override_cfg);
        assert_eq!(merged.search_limit, 100);
        assert_eq!(merged.deadband_threshold, 0.7);
    }

    #[test]
    fn test_from_env_uses_defaults_when_no_env_vars() {
        std::env::remove_var("PLATO_DEADBAND_THRESHOLD");
        std::env::remove_var("PLATO_SEARCH_LIMIT");
        std::env::remove_var("PLATO_DECAY_RATE");
        std::env::remove_var("PLATO_MAX_ROOMS");

        let cfg = from_env();
        let defaults = default_config();
        assert_eq!(cfg.deadband_threshold, defaults.deadband_threshold);
        assert_eq!(cfg.search_limit, defaults.search_limit);
        assert_eq!(cfg.decay_rate, defaults.decay_rate);
        assert_eq!(cfg.max_rooms, defaults.max_rooms);
    }

    #[test]
    fn test_from_env_reads_plato_search_limit() {
        std::env::remove_var("PLATO_DEADBAND_THRESHOLD");
        std::env::remove_var("PLATO_DECAY_RATE");
        std::env::remove_var("PLATO_MAX_ROOMS");
        std::env::set_var("PLATO_SEARCH_LIMIT", "42");
        let cfg = from_env();
        assert_eq!(cfg.search_limit, 42);
        std::env::remove_var("PLATO_SEARCH_LIMIT");
    }

    #[test]
    fn test_validate_max_rooms_zero() {
        let mut cfg = default_config();
        cfg.max_rooms = 0;
        let errors = validate(&cfg);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("max_rooms")));
    }

    #[test]
    fn test_validate_decay_rate_above_one() {
        let mut cfg = default_config();
        cfg.decay_rate = 1.5;
        let errors = validate(&cfg);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("decay_rate")));
    }
}
