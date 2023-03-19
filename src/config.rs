//! Defines mithril's `mithril.toml` config file structure and content.

use crate::metric::MetricConfig;
use crate::stratum::stratum_data::PoolConfig;
use crate::worker::worker_pool::WorkerConfig;

use serde::{de, Deserialize, Deserializer};
use std::{fs::read_to_string, io, path::Path};

/// Config file location.
pub const CONFIG_FILE_NAME: &str = "mithril.toml";

/// TOML config definition. Located at [`CONFIG_FILE_NAME`].
///
/// Each field is its own TOML key like this:
/// ```toml
/// [pool]
/// ```
///
/// Each field within a substruct is a subkey like this:
/// ```toml
/// [pool]
/// pool_address = "example.com:1111"
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MithrilConfig {
    pub pool: PoolConfig,
    pub worker: WorkerConfig,
    /// Performance measurements
    pub metric: MetricConfig,
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

/// Sets the donation settings
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DonationConfig {
    /// `0` disables donation mining, `1..100` will make your miner mine on
    /// the project's address for a portion of the time.
    #[serde(deserialize_with = "str_to_percentage")]
    pub percentage: f64,
}

fn str_to_percentage<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let num: f64 = Deserialize::deserialize(deserializer)?;
    // ceil because 100.1 is invalid
    match num.ceil() as isize {
        i if (0..=100).contains(&i) => Ok(num),
        _ => Err(de::Error::invalid_value(
            de::Unexpected::Float(num),
            &"within 0 to 100",
        )),
    }
}

impl MithrilConfig {
    pub fn from_file(path: &Path) -> anyhow::Result<MithrilConfig> {
        let conf = read_to_string(path)
            .map_err(|e| MithrilConfigError::ReadingFile(path.to_string_lossy().to_string(), e))?;

        let toml: toml::Value = toml::from_str(&conf)?;
        Ok(Self::deserialize(toml)?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    const DEFAULT_CONFIG_FILE: &'static str = include_str!("../default_config.toml");

    #[test]
    fn config_parsing() {
        let toml: toml::Value = toml::from_str(DEFAULT_CONFIG_FILE).unwrap();
        let parsed = MithrilConfig::deserialize(toml).unwrap();
        assert_eq!(
            parsed,
            MithrilConfig {
                pool: PoolConfig::new("xmrpool.eu:3333", "wallet", "x"),
                worker: WorkerConfig::new(8, true, 15, "./bandit.log"),
                metric: MetricConfig::new(false, 100, 60, "/path/to/hash/report/file.csv"),
                donation: DonationConfig { percentage: 2.5 }
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
        let conf = format!("percentage = {}", percentage);
        let toml: toml::Value = toml::from_str(&conf).unwrap();
        let parsed = DonationConfig::deserialize(toml);

        if valid {
            assert_eq!(parsed.unwrap(), DonationConfig { percentage });
        } else {
            assert!(parsed.is_err());
        }
    }
}
