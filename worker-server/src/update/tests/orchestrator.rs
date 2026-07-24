use crate::update::tests::support::{
    checksum, release_archive, release_routes, FakeRestarter, TestServer,
};
use crate::update::{AutomaticUpdater, UpdateOutcome, VERSION_FILE};

const TARGET: &str = "x86_64-unknown-linux-gnu";

#[tokio::test]
async fn applies_one_verified_release_pair_then_restarts_service() {
    let archive = release_archive(b"new-cli", b"new-worker");
    let checksum = checksum(&archive);
    let server = TestServer::start(3, |base_url| {
        release_routes(base_url, "v2.0.0", TARGET, archive, checksum)
    })
    .await;
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    write_installed_pair(temporary.path());
    let restarter = FakeRestarter::default();
    let updater = updater(&server, temporary.path(), restarter.clone());

    let outcome = updater.check_and_apply().await;

    assert!(matches!(
        outcome,
        UpdateOutcome::Applied {
            previous_version,
            target_version,
            ..
        } if previous_version == "v1.0.0" && target_version == "v2.0.0"
    ));
    assert_eq!(restarter.calls(), 1);
    assert_eq!(read(temporary.path(), "vulcanum"), b"new-cli");
    assert_eq!(read(temporary.path(), "vulcanum-server"), b"new-worker");
    assert_eq!(read(temporary.path(), VERSION_FILE), b"v2.0.0\n");
}

#[tokio::test]
async fn current_release_is_a_no_op_without_service_restart() {
    let archive = release_archive(b"unused-cli", b"unused-worker");
    let checksum = checksum(&archive);
    let server = TestServer::start(1, |base_url| {
        release_routes(base_url, "v1.0.0", TARGET, archive, checksum)
    })
    .await;
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    write_installed_pair(temporary.path());
    let restarter = FakeRestarter::default();
    let updater = updater(&server, temporary.path(), restarter.clone());

    let outcome = updater.check_and_apply().await;

    assert_eq!(
        outcome,
        UpdateOutcome::UpToDate {
            version: "v1.0.0".to_owned()
        }
    );
    assert_eq!(restarter.calls(), 0);
    assert_eq!(read(temporary.path(), "vulcanum"), b"old-cli");
    assert_eq!(read(temporary.path(), "vulcanum-server"), b"old-worker");
}

#[tokio::test]
async fn checksum_failure_retains_pair_and_reports_target_version() {
    let archive = release_archive(b"new-cli", b"new-worker");
    let server = TestServer::start(3, |base_url| {
        release_routes(
            base_url,
            "v2.0.0",
            TARGET,
            archive,
            format!("{}  archive.tar.gz\n", "0".repeat(64)).into_bytes(),
        )
    })
    .await;
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    write_installed_pair(temporary.path());
    let restarter = FakeRestarter::default();
    let updater = updater(&server, temporary.path(), restarter.clone());

    let outcome = updater.check_and_apply().await;

    assert!(matches!(
        outcome,
        UpdateOutcome::Failed {
            target_version: Some(target),
            error,
            ..
        } if target == "v2.0.0" && error.contains("checksum verification failed")
    ));
    assert_eq!(restarter.calls(), 0);
    assert_eq!(read(temporary.path(), "vulcanum"), b"old-cli");
    assert_eq!(read(temporary.path(), "vulcanum-server"), b"old-worker");
}

#[tokio::test]
async fn network_failure_is_reported_without_a_target_or_restart() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    write_installed_pair(temporary.path());
    let restarter = FakeRestarter::default();
    let updater = AutomaticUpdater::new(
        "http://[::1/latest".to_owned(),
        temporary.path().to_path_buf(),
        "v1.0.0".to_owned(),
        TARGET.to_owned(),
        restarter.clone(),
    )
    .expect("updater should be created");

    let outcome = updater.check_and_apply().await;

    assert!(matches!(
        outcome,
        UpdateOutcome::Failed {
            target_version: None,
            error,
            ..
        } if error.contains("failed to request the latest GitHub release")
    ));
    assert_eq!(restarter.calls(), 0);
    assert_eq!(read(temporary.path(), "vulcanum"), b"old-cli");
    assert_eq!(read(temporary.path(), "vulcanum-server"), b"old-worker");
}

fn updater(
    server: &TestServer,
    install_dir: &std::path::Path,
    restarter: FakeRestarter,
) -> AutomaticUpdater<FakeRestarter> {
    AutomaticUpdater::new(
        format!("{}/latest", server.base_url),
        install_dir.to_path_buf(),
        "v1.0.0".to_owned(),
        TARGET.to_owned(),
        restarter,
    )
    .expect("updater should be created")
}

fn write_installed_pair(install_dir: &std::path::Path) {
    std::fs::write(install_dir.join("vulcanum"), b"old-cli").expect("CLI should be written");
    std::fs::write(install_dir.join("vulcanum-server"), b"old-worker")
        .expect("worker should be written");
    std::fs::write(install_dir.join(VERSION_FILE), b"v1.0.0").expect("version should be written");
}

fn read(directory: &std::path::Path, name: &str) -> Vec<u8> {
    std::fs::read(directory.join(name)).expect("file should be readable")
}
