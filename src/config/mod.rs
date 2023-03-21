//! Defines mithril's TOML config file structure and content, located at [`CONFIG_FILE_NAME`].
//! See [`MithrilConfig`] for the details on how to write your TOML file.

use crate::{
    metric::MetricConfig, stratum::stratum_data::PoolConfig, worker::worker_pool::WorkerConfig,
};
use anyhow::Context;
use serde::Deserialize;
use std::{fs::read_to_string, io, str::FromStr};

mod donation;
pub use donation::DonationConfig;

/// Config file location.
pub const CONFIG_FILE_NAME: &str = "mithril.toml";

/// TOML config definition. Located at [`CONFIG_FILE_NAME`].
///
/// Each field is its own TOML key like this:
/// ```toml
/// [foo] # Just an example, there is no actual foo in the file
/// ```
///
/// Each field within a field (aka foo.bar) is a subkey like this:
/// ```toml
/// [foo]
/// bar = "example"
/// ```
///
/// You can also represent fields compactly like this:
/// ```toml
/// foo = { bar = "example" }
/// ```
///
/// Click on e.g. [`PoolConfig`] below to view details.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MithrilConfig {
    pub pool: PoolConfig,

    pub worker: WorkerConfig,

    /// Performance metrics
    pub metric: MetricConfig,

    #[serde(default)]
    pub donation: DonationConfig,
}

#[derive(Error, Debug)]
pub enum MithrilConfigError {
    #[error(
        r#"Could not read content of config file: "{0}".
Details: {1}"#
    )]
    ReadingFile(String, io::Error),

    #[error("Failed to parse TOML file: {0}")]
    ParsingError(toml::de::Error),
}

impl From<toml::de::Error> for MithrilConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::ParsingError(e)
    }
}

impl MithrilConfig {
    pub fn from_file(path: &str) -> anyhow::Result<MithrilConfig> {
        let conf = read_to_string(path).map_err(|e| {
            debug!("While opening config file {path}: {e:#?}",);
            MithrilConfigError::ReadingFile(path.to_string(), e)
        })?;

        let error_context = || format!("Parsing config file {path}.");
        let toml: toml::Value = toml::from_str(&conf).with_context(error_context)?;
        Ok(Self::deserialize(toml).with_context(error_context)?)
    }
}

impl FromStr for MithrilConfig {
    type Err = MithrilConfigError;
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Ok(MithrilConfig::deserialize(toml::from_str::<toml::Value>(
            source,
        )?)?)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::worker::worker_pool::AutoTuneConfig;

    use super::*;

    #[test_case(MithrilConfig::from_str(include_str!("../../default_config.toml")).unwrap(); "from string")]
    #[test_case(MithrilConfig::from_file("default_config.toml").unwrap(); "from file")]
    fn parse_config(parsed: MithrilConfig) {
        assert_eq!(
            parsed,
            MithrilConfig {
                pool: PoolConfig::new("example.com", "x", "wallet"),
                worker: WorkerConfig::new(8, AutoTuneConfig::new(15, "bandit.log")),
                metric: MetricConfig::new(true, 100, 60, "mithril_metrics.csv"),
                donation: DonationConfig(2.5),
            }
        );
    }

    #[test_case(0.0, true)]
    #[test_case(100.0, true)]
    #[test_case(52.0376, true)]
    #[test_case(-1.83, false; "negative 1.83")]
    #[test_case(1000.1, false)]
    #[test_case(100.0001, false)]
    fn percentage_is_validated(percentage: f64, valid: bool) {
        let conf = format!("donation = {percentage}");
        let toml: toml::Value = toml::from_str(&conf).unwrap();
        let parsed = DonationConfig::deserialize(toml["donation"].clone());

        if valid {
            assert_eq!(parsed.unwrap(), DonationConfig(percentage));
        } else {
            assert!(parsed.is_err());
        }
    }
}
