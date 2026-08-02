use std::ffi::OsString;
use std::sync::{Mutex, MutexGuard};

use serde_json::{json, Map};

use tempfile::TempDir;
use vulcanum_shared::config::{load_config, save_config, WorkerConfig};

use crate::commands::worker_updates;

static HOME_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn newly_created_config_enables_updates_with_default_interval() {
    let _home = TestHome::new();

    let config = load_config().expect("default worker config should be created");
    let saved = load_config().expect("saved worker config should load");

    assert!(config.auto_update_enabled);
    assert_eq!(config.update_check_interval_secs, 24 * 60 * 60);
    assert!(saved.auto_update_enabled);
    assert_eq!(saved.update_check_interval_secs, 24 * 60 * 60);
}

#[test]
fn disable_preserves_unrelated_worker_configuration() {
    let _home = TestHome::new();
    let original = configured_worker(true);
    save_config(&original).expect("worker config should be saved");

    worker_updates::run(false).expect("worker updates should be disabled");

    let updated = load_config().expect("updated worker config should load");
    assert!(!updated.auto_update_enabled);
    assert_unrelated_fields_are_preserved(&updated, &original);
}

#[test]
fn explicit_opt_out_remains_disabled_until_enabled() {
    let _home = TestHome::new();
    let original = configured_worker(false);
    save_config(&original).expect("worker config should be saved");

    let opted_out = load_config().expect("opted-out worker config should load");
    assert!(!opted_out.auto_update_enabled);

    worker_updates::run(true).expect("worker updates should be enabled");

    let updated = load_config().expect("updated worker config should load");
    assert!(updated.auto_update_enabled);
    assert_unrelated_fields_are_preserved(&updated, &original);
}

fn configured_worker(auto_update_enabled: bool) -> WorkerConfig {
    let mut extra = Map::new();
    extra.insert("future_setting".to_owned(), json!({ "enabled": true }));

    WorkerConfig {
        harness: "kata".to_owned(),
        image: "registry.example/vulcanum/agent:pinned".to_owned(),
        log_format: Some("json".to_owned()),
        debug: true,
        poll_interval_secs: 17,
        auto_update_enabled,
        update_check_interval_secs: 3_600,
        extra,
    }
}

fn assert_unrelated_fields_are_preserved(actual: &WorkerConfig, expected: &WorkerConfig) {
    assert_eq!(actual.harness, expected.harness);
    assert_eq!(actual.image, expected.image);
    assert_eq!(actual.log_format, expected.log_format);
    assert_eq!(actual.debug, expected.debug);
    assert_eq!(actual.poll_interval_secs, expected.poll_interval_secs);
    assert_eq!(
        actual.update_check_interval_secs,
        expected.update_check_interval_secs
    );
    assert_eq!(actual.extra, expected.extra);
}

struct TestHome {
    previous_home: Option<OsString>,
    _temporary: TempDir,
    _guard: MutexGuard<'static, ()>,
}

impl TestHome {
    fn new() -> Self {
        let guard = HOME_LOCK
            .lock()
            .expect("HOME override lock should not be poisoned");
        let temporary = tempfile::tempdir().expect("temporary HOME should be created");
        let previous_home = std::env::var_os("HOME");
        std::env::set_var("HOME", temporary.path());

        Self {
            previous_home,
            _temporary: temporary,
            _guard: guard,
        }
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        match self.previous_home.as_ref() {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
    }
}
