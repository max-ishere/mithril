use mithril::config::MithrilConfig;
use std::time::{Duration, Instant};

#[test] //Bugfix test, there should be some "room" so that this value can be added to a time instant
fn test_disabled_metric_value_should_be_addable_to_now() {
    let config = MithrilConfig::from_file("./default_config.toml").unwrap();

    let now = Instant::now();
    let _ = now + Duration::from_secs(config.metric.sample_interval_seconds);
    //Ok if it doesn't panic
}
