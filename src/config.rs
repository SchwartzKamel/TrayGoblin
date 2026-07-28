use std::{fmt, path::PathBuf, time::Duration};

use serde::Deserialize;

/// Lower bound of the accepted polling interval, in milliseconds.
pub const MIN_POLL_INTERVAL_MS: u64 = 500;
/// Upper bound of the accepted polling interval, in milliseconds.
pub const MAX_POLL_INTERVAL_MS: u64 = 10_000;
/// Default polling interval used when none is configured.
pub const DEFAULT_POLL_INTERVAL_MS: u64 = 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    PollIntervalOutOfRange(u64),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PollIntervalOutOfRange(value) => write!(
                f,
                "poll interval {value}ms is outside the accepted {MIN_POLL_INTERVAL_MS}-{MAX_POLL_INTERVAL_MS}ms range"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Content-free monitor configuration: only a session-root override and a
/// bounded polling cadence. Never carries prompt, credential, or token data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonitorConfig {
    session_root: Option<PathBuf>,
    poll_interval_ms: u64,
}

#[derive(Debug, Default, Deserialize)]
struct MonitorConfigDocument {
    #[serde(default, alias = "sessionRoot")]
    session_root: Option<PathBuf>,
    #[serde(default, alias = "pollIntervalMs")]
    poll_interval_ms: Option<u64>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            session_root: None,
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
        }
    }
}

impl MonitorConfig {
    /// Builds a configuration, rejecting a poll interval outside 500-10,000 ms.
    pub fn new(session_root: Option<PathBuf>, poll_interval_ms: u64) -> Result<Self, ConfigError> {
        if !(MIN_POLL_INTERVAL_MS..=MAX_POLL_INTERVAL_MS).contains(&poll_interval_ms) {
            return Err(ConfigError::PollIntervalOutOfRange(poll_interval_ms));
        }

        Ok(Self {
            session_root,
            poll_interval_ms,
        })
    }

    /// Parses a JSON configuration document. Unknown fields are ignored so
    /// forward-compatible config files remain non-fatal; an out-of-range
    /// poll interval is still a typed, explicit error.
    pub fn parse(input: &str) -> Result<Self, ConfigParseError> {
        let document: MonitorConfigDocument =
            serde_json::from_str(input).map_err(ConfigParseError::InvalidJson)?;

        let poll_interval_ms = document
            .poll_interval_ms
            .unwrap_or(DEFAULT_POLL_INTERVAL_MS);

        Self::new(document.session_root, poll_interval_ms).map_err(ConfigParseError::Invalid)
    }

    pub fn session_root(&self) -> Option<&PathBuf> {
        self.session_root.as_ref()
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms)
    }

    pub fn poll_interval_ms(&self) -> u64 {
        self.poll_interval_ms
    }
}

#[derive(Debug)]
pub enum ConfigParseError {
    InvalidJson(serde_json::Error),
    Invalid(ConfigError),
}

impl fmt::Display for ConfigParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(f, "invalid configuration JSON: {error}"),
            Self::Invalid(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ConfigParseError {}

/// Best-effort default Copilot CLI session-state root, matching
/// `%USERPROFILE%\.copilot\session-state` on Windows. Used only when no
/// override is configured; callers should still handle a missing directory.
pub fn default_session_root() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
    Some(PathBuf::from(home).join(".copilot").join("session-state"))
}

#[cfg(test)]
mod tests {
    use super::{
        ConfigError, ConfigParseError, MAX_POLL_INTERVAL_MS, MIN_POLL_INTERVAL_MS, MonitorConfig,
    };

    // This protects the documented 500-10,000 ms acceptance boundary at its edges.
    #[test]
    fn accepts_the_documented_interval_boundaries() {
        assert!(MonitorConfig::new(None, MIN_POLL_INTERVAL_MS).is_ok());
        assert!(MonitorConfig::new(None, MAX_POLL_INTERVAL_MS).is_ok());
    }

    // This is the explicit-error contract: out-of-range intervals never silently clamp.
    #[test]
    fn rejects_interval_below_the_minimum() {
        let error = MonitorConfig::new(None, MIN_POLL_INTERVAL_MS - 1).unwrap_err();
        assert_eq!(
            error,
            ConfigError::PollIntervalOutOfRange(MIN_POLL_INTERVAL_MS - 1)
        );
    }

    // This is the explicit-error contract on the upper bound of the polling cadence.
    #[test]
    fn rejects_interval_above_the_maximum() {
        let error = MonitorConfig::new(None, MAX_POLL_INTERVAL_MS + 1).unwrap_err();
        assert_eq!(
            error,
            ConfigError::PollIntervalOutOfRange(MAX_POLL_INTERVAL_MS + 1)
        );
    }

    // This protects the default cadence used when a config file omits the interval.
    #[test]
    fn parses_session_root_override_with_default_interval() {
        let config = MonitorConfig::parse(r#"{"sessionRoot":"C:/fixture/session-state"}"#)
            .expect("valid JSON with a known field should parse");

        assert_eq!(config.poll_interval_ms(), super::DEFAULT_POLL_INTERVAL_MS);
        assert_eq!(
            config.session_root(),
            Some(&std::path::PathBuf::from("C:/fixture/session-state"))
        );
    }

    // This keeps forward-compatible config files (extra unknown keys) from being fatal.
    #[test]
    fn ignores_unknown_config_fields() {
        let config = MonitorConfig::parse(r#"{"pollIntervalMs":750,"futureField":"changed"}"#)
            .expect("unknown fields should be ignored");

        assert_eq!(config.poll_interval_ms(), 750);
    }

    // This proves an invalid interval surfaces as a typed parse error, not a panic or clamp.
    #[test]
    fn parse_rejects_out_of_range_interval() {
        let error = MonitorConfig::parse(r#"{"pollIntervalMs":100}"#).unwrap_err();
        assert!(matches!(
            error,
            ConfigParseError::Invalid(ConfigError::PollIntervalOutOfRange(100))
        ));
    }
}
