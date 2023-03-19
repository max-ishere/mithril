extern crate mithril;

use mithril::config::MithrilConfig;

use std::path::Path;
use std::time::{Duration, Instant};

#[test]
fn test_read_default_config() {
    let config = read_default_config();

    assert_eq!(config.pool.pool_address, "xmrpool.eu:3333");
    assert_eq!(config.pool.wallet_address, "wallet");
    assert_eq!(config.pool.pool_password, "x");

    assert_eq!(config.worker.num_threads, 8);
    assert_eq!(config.worker.auto_tune, true);
    assert_eq!(config.worker.auto_tune_interval_minutes, 15);
    assert_eq!(config.worker.auto_tune_log, "./bandit.log");

    assert_eq!(config.metric.enabled, false);
    assert_eq!(config.metric.resolution, std::u32::MAX as u64);
    assert_eq!(config.metric.sample_interval_seconds, std::u32::MAX as u64);
    assert_eq!(config.metric.report_file, "/dev/null");

    assert_eq!(config.donation.percentage, 2.5);
}

#[test] //Bugfix test, there should be some "room" so that this value can be added to a time instant
fn test_disabled_metric_value_should_be_addable_to_now() {
    let config = read_default_config();

    let now = Instant::now();
    let _ = now + Duration::from_secs(config.metric.sample_interval_seconds);
    //Ok if it doesn't panic
}

//helper

fn read_default_config() -> MithrilConfig {
    let path = &format!("{}{}", "./", "default_config.toml");
    return MithrilConfig::from_file(Path::new(path)).unwrap();
}
