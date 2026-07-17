use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::state::app::AppSession;
use crate::state::file::{load_json, save_json};

#[test]
fn app_session_round_trips_through_restricted_json_helper() {
    let temp_dir = temp_path("round-trip");
    let path = temp_dir.join("nested").join("app.json");
    let expected = app_session();

    save_json(&path, &expected).expect("session should save");
    let actual = load_json(&path).expect("session should load");

    assert_eq!(actual, Some(expected));
    std::fs::remove_dir_all(temp_dir).expect("temporary directory should be removed");
}

#[test]
fn legacy_app_session_without_team_id_loads_without_pin() {
    let session: AppSession = serde_json::from_str(
        r#"{
            "instance_url":"https://vulcanum.example",
            "access_token":"access-token",
            "refresh_token":"refresh-token",
            "refresh_expires_at":"2030-01-02T03:04:05Z"
        }"#,
    )
    .expect("legacy session should deserialize");

    assert_eq!(session.team_id, None);
}

#[test]
fn pinned_team_survives_save_and_load() {
    let temp_dir = temp_path("pinned-team");
    let path = temp_dir.join("app.json");
    let mut expected = app_session();
    expected.team_id = Some(uuid::Uuid::from_u128(42));

    save_json(&path, &expected).expect("session should save");
    let actual = load_json(&path).expect("session should load");

    assert_eq!(actual, Some(expected));
    std::fs::remove_dir_all(temp_dir).expect("temporary directory should be removed");
}

#[cfg(unix)]
#[test]
fn public_save_state_uses_app_path_and_repairs_permissions() {
    use std::ffi::OsString;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::MutexGuard;

    use crate::state::app::{load_state, save_state};
    use crate::state::file::HOME_LOCK;

    struct HomeOverride {
        previous_home: Option<OsString>,
        temp_dir: PathBuf,
        _lock: MutexGuard<'static, ()>,
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

    let lock = HOME_LOCK
        .lock()
        .expect("HOME override lock should not be poisoned");
    let temp_dir = temp_path("home");
    std::fs::create_dir_all(&temp_dir).expect("temporary HOME should be created");
    let home = HomeOverride {
        previous_home: std::env::var_os("HOME"),
        temp_dir,
        _lock: lock,
    };
    std::env::set_var("HOME", &home.temp_dir);
    let path = home.temp_dir.join(".vulcanum").join("app.json");
    std::fs::create_dir_all(path.parent().expect("app path should have parent"))
        .expect("state directory should be created");
    std::fs::write(&path, b"{}").expect("placeholder should be written");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o666))
        .expect("placeholder should have loose permissions");
    let expected = app_session();

    save_state(&expected).expect("public state save should succeed");

    assert_eq!(
        load_state().expect("public state load should succeed"),
        Some(expected)
    );
    let mode = std::fs::metadata(path)
        .expect("saved state should exist")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

fn app_session() -> AppSession {
    AppSession {
        instance_url: "https://vulcanum.example".to_owned(),
        access_token: "access-token".to_owned(),
        refresh_token: "refresh-token".to_owned(),
        refresh_expires_at: DateTime::parse_from_rfc3339("2030-01-02T03:04:05Z")
            .expect("fixed timestamp should parse")
            .with_timezone(&Utc),
        team_id: None,
    }
}

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vulcanum-app-state-{name}-{}", std::process::id()))
}
