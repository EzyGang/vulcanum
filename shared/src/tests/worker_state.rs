#![cfg(unix)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::MutexGuard;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::state::file::HOME_LOCK;
use crate::state::worker::{save_state, WorkerState};

struct HomeOverride {
    previous_home: Option<OsString>,
    temp_dir: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl HomeOverride {
    fn new(name: &str) -> Self {
        let lock = HOME_LOCK
            .lock()
            .expect("HOME override lock should not be poisoned");
        let temp_dir = std::env::temp_dir().join(format!(
            "vulcanum-worker-state-home-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("temporary HOME should be created");

        let previous_home = std::env::var_os("HOME");
        std::env::set_var("HOME", &temp_dir);

        Self {
            previous_home,
            temp_dir,
            _lock: lock,
        }
    }
}

impl Drop for HomeOverride {
    fn drop(&mut self) {
        match self.previous_home.as_ref() {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn save_state_repairs_existing_state_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let home = HomeOverride::new("loose-permissions");
    let state_path = home.temp_dir.join(".vulcanum").join("worker.json");
    std::fs::create_dir_all(state_path.parent().expect("state path should have parent"))
        .expect("state directory should be created");
    std::fs::write(&state_path, b"{}").expect("placeholder state should be written");
    std::fs::set_permissions(&state_path, std::fs::Permissions::from_mode(0o666))
        .expect("placeholder state should start with loose permissions");

    let state = worker_state();

    save_state(&state).expect("state should save successfully");

    let mode = std::fs::metadata(&state_path)
        .expect("saved state should exist")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);

    let saved: WorkerState = serde_json::from_str(
        &std::fs::read_to_string(&state_path).expect("saved state should be readable"),
    )
    .expect("saved state should be valid JSON");
    assert_eq!(saved.worker_id, state.worker_id);
    assert_eq!(saved.access_token, state.access_token);
}

fn worker_state() -> WorkerState {
    WorkerState {
        worker_id: Uuid::from_u128(0x12345678123456781234567812345678),
        instance_url: "https://vulcanum.example".to_owned(),
        access_token: "access-token".to_owned(),
        refresh_token: "refresh-token".to_owned(),
        expires_at: DateTime::parse_from_rfc3339("2026-07-07T00:00:00Z")
            .expect("fixed timestamp should parse")
            .with_timezone(&Utc),
        max_concurrent_jobs: 2,
    }
}
