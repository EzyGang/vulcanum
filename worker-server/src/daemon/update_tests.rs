use std::time::Duration;

use vulcanum_shared::config::WorkerConfig;

use super::{
    automatic_update_interval, MAX_UPDATE_CHECK_INTERVAL_SECS, MIN_UPDATE_CHECK_INTERVAL_SECS,
};

#[test]
fn default_updates_use_the_default_interval() {
    let config = WorkerConfig::default();

    assert!(config.auto_update_enabled);
    assert_eq!(
        automatic_update_interval(&config).expect("default updates should schedule"),
        Duration::from_secs(config.update_check_interval_secs)
    );
}

#[test]
fn disabled_updates_ignore_unusable_interval() {
    let config = WorkerConfig {
        auto_update_enabled: false,
        update_check_interval_secs: u64::MAX,
        ..WorkerConfig::default()
    };

    assert_eq!(
        automatic_update_interval(&config).expect("disabled updates should not schedule"),
        Duration::ZERO
    );
}

#[test]
fn enabled_updates_require_bounded_nonzero_interval() {
    for interval in [0, MIN_UPDATE_CHECK_INTERVAL_SECS - 1, u64::MAX] {
        let config = WorkerConfig {
            auto_update_enabled: true,
            update_check_interval_secs: interval,
            ..WorkerConfig::default()
        };

        assert!(automatic_update_interval(&config).is_err());
    }
}

#[test]
fn enabled_updates_accept_interval_boundaries() {
    for interval in [
        MIN_UPDATE_CHECK_INTERVAL_SECS,
        MAX_UPDATE_CHECK_INTERVAL_SECS,
    ] {
        let config = WorkerConfig {
            auto_update_enabled: true,
            update_check_interval_secs: interval,
            ..WorkerConfig::default()
        };

        assert_eq!(
            automatic_update_interval(&config).expect("bounded interval should schedule"),
            Duration::from_secs(interval)
        );
    }
}
