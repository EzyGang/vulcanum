use serde_json::json;

use crate::config::WorkerConfig;

#[test]
fn worker_config_defaults_to_verified_updates() {
    let config = WorkerConfig::default();

    assert!(config.auto_update_enabled);
    assert_eq!(config.update_check_interval_secs, 24 * 60 * 60);
}

#[test]
fn deserialization_preserves_legacy_opt_out_for_missing_or_explicit_values() {
    let defaulted: WorkerConfig =
        serde_json::from_value(json!({})).expect("minimal config should deserialize");
    let opted_out: WorkerConfig = serde_json::from_value(json!({
        "auto_update_enabled": false
    }))
    .expect("explicit worker opt-out should deserialize");

    assert!(!defaulted.auto_update_enabled);
    assert!(!opted_out.auto_update_enabled);
}
