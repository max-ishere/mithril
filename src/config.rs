//! Defines mithril's `mithril.toml` config file structure and content.

use crate::metric::MetricConfig;
use crate::stratum::stratum_data::PoolConfig;
use crate::worker::worker_pool::WorkerConfig;

use config::{Config, ConfigError, File};
use serde::{
    de::{self, Expected},
    Deserialize, Deserializer,
};
use std::{fmt::Display, path::Path, str::FromStr};

pub const CONFIG_FILE_NAME: &str = "mithril.toml";

/// `mithril.toml` definition
///
/// Each field is its own TOML key like this:
/// ```toml
/// [pool]
/// ```
///
/// Each field within a substruct is a subkey like this:
/// ```toml
/// [pool]
/// pool_address = "https://example.com:1111"
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MithrilConfig {
    pub pool: PoolConfig,
    pub worker: WorkerConfig,
    /// Performance measurements
    pub metric: MetricConfig,
    pub donation: DonationConfig,
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
    pub fn from_file(conf_file: &Path) -> Result<MithrilConfig, config::ConfigError> {
        // TODO: Rewrite this function using Self::deserialize()
        let config = parse_conf(conf_file)?;

        let pool_conf = pool_config(&config)?;
        let worker_conf = worker_config(&config)?;
        let metric_conf = metric_config(&config)?;
        let donation_conf = donation_config(&config)?;

        Ok(MithrilConfig {
            pool: pool_conf,
            worker: worker_conf,
            metric: metric_conf,
            donation: donation_conf,
        })
    }
}

fn donation_config(conf: &Config) -> Result<DonationConfig, ConfigError> {
    let percentage = conf.get_float("donation.percentage")?;
    Ok(DonationConfig { percentage })
}

fn pool_config(conf: &Config) -> Result<PoolConfig, ConfigError> {
    let pool_address = conf.get_string("pool.pool_address")?;
    let wallet_address = conf.get_string("pool.wallet_address")?;
    let pool_password = conf.get_string("pool.pool_password")?;
    Ok(PoolConfig {
        pool_address,
        wallet_address,
        pool_password,
    })
}

fn worker_config(conf: &Config) -> Result<WorkerConfig, ConfigError> {
    let num_threads = conf.get_int("worker.num_threads")?;
    if num_threads <= 0 {
        return Err(ConfigError::Message(
            "num_threads has to be > 0".to_string(),
        ));
    }

    let auto_tune = conf.get_bool("worker.auto_tune")?;

    let auto_tune_interval_minutes = conf.get_int("worker.auto_tune_interval_minutes")?;
    if auto_tune_interval_minutes <= 0 {
        return Err(ConfigError::Message(
            "auto_tune_interval_minutes has to be > 0".to_string(),
        ));
    }

    let auto_tune_log = conf.get_string("worker.auto_tune_log")?;

    Ok(WorkerConfig {
        num_threads: num_threads as u64,
        auto_tune,
        auto_tune_interval_minutes: auto_tune_interval_minutes as u64,
        auto_tune_log,
    })
}

fn metric_config(conf: &Config) -> Result<MetricConfig, ConfigError> {
    let enabled = conf.get_bool("metric.enabled")?;
    if enabled {
        let resolution = get_u64_no_zero(conf, "metric.resolution")?;
        let sample_interval_seconds = get_u64_no_zero(conf, "metric.sample_interval_seconds")?;
        let report_file = conf.get_string("metric.report_file")?;
        Ok(MetricConfig {
            enabled,
            resolution,
            sample_interval_seconds,
            report_file,
        })
    } else {
        Ok(MetricConfig {
            enabled: false,
            resolution: std::u32::MAX as u64,
            sample_interval_seconds: std::u32::MAX as u64,
            report_file: "/dev/null".to_string(),
        })
    }
}

fn get_u64_no_zero(conf: &Config, field: &str) -> Result<u64, ConfigError> {
    let val = conf.get_int(field)?;
    if val <= 0 {
        return Err(ConfigError::Message(format!("{} has to be > 0", field)));
    }
    Ok(val as u64)
}

fn parse_conf(conf_file: &Path) -> Result<Config, ConfigError> {
    if conf_file.exists() {
        let mut conf = Config::default();
        conf.merge(File::with_name(
            &conf_file
                .file_name()
                .ok_or_else(|| ConfigError::Message(conf_file.to_string_lossy().to_string()))?
                .to_string_lossy(),
        ))?;
        return Ok(conf);
    }
    Err(ConfigError::Message("config file not found".to_string()))
}

pub fn donation_conf() -> PoolConfig {
    PoolConfig {
        pool_address: "xmrpool.eu:3333".to_string(),
        pool_password: "x".to_string(),
        wallet_address: "48y3RCT5SzSS4jumHm9rRL91eWWzd6xcVGSCF1KUZGWYJ6npqwFxHee4xkLLNUqY4NjiswdJhxFALeRqzncHoToeJMg2bhL".to_string()
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
