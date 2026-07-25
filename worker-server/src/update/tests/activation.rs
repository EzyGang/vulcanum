use std::cell::Cell;

use crate::update::activation::{
    activate_pair, activate_pair_with, confirm_pending_activation, prepare_startup,
    recover_interrupted_activation, rollback_pending_activation, StartupActivation,
};
use crate::update::VERSION_FILE;

#[test]
fn activates_both_binaries_and_retains_previous_pair() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let install_dir = temporary.path().join("install");
    let staging_dir = temporary.path().join("staging");
    std::fs::create_dir_all(&install_dir).expect("install directory should be created");
    std::fs::create_dir_all(&staging_dir).expect("staging directory should be created");
    write_pair(&install_dir, b"old-cli", b"old-worker", "v1.0.0");
    write_pair(&staging_dir, b"new-cli", b"new-worker", "v2.0.0");

    let rollback_dir =
        activate_pair(&staging_dir, &install_dir, "v1.0.0").expect("release pair should activate");

    assert_eq!(read(&install_dir, "vulcanum"), b"new-cli");
    assert_eq!(read(&install_dir, "vulcanum-server"), b"new-worker");
    assert_eq!(read(&install_dir, VERSION_FILE), b"v2.0.0");
    assert_eq!(read(&rollback_dir, "vulcanum"), b"old-cli");
    assert_eq!(read(&rollback_dir, "vulcanum-server"), b"old-worker");
    assert_eq!(read(&rollback_dir, VERSION_FILE), b"v1.0.0");
}

#[test]
fn commits_only_after_replacement_startup_is_confirmed() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let install_dir = temporary.path().join("install");
    let staging_dir = temporary.path().join("staging");
    std::fs::create_dir_all(&install_dir).expect("install directory should be created");
    std::fs::create_dir_all(&staging_dir).expect("staging directory should be created");
    write_pair(&install_dir, b"old-cli", b"old-worker", "v1.0.0");
    write_pair(&staging_dir, b"new-cli", b"new-worker", "v2.0.0");

    let rollback_dir =
        activate_pair(&staging_dir, &install_dir, "v1.0.0").expect("release pair should activate");
    assert_eq!(
        prepare_startup(&install_dir).expect("pending startup should prepare"),
        StartupActivation::Pending(rollback_dir.clone())
    );
    assert_eq!(read(&install_dir, "vulcanum"), b"new-cli");

    confirm_pending_activation(&install_dir, &rollback_dir)
        .expect("healthy startup should commit activation");

    assert_eq!(
        prepare_startup(&install_dir).expect("committed startup should be clean"),
        StartupActivation::Clean
    );
    assert_eq!(read(&install_dir, "vulcanum"), b"new-cli");
    assert_eq!(read(&install_dir, "vulcanum-server"), b"new-worker");
}

#[test]
fn restores_previous_pair_when_replacement_crashes_during_startup() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let install_dir = temporary.path().join("install");
    let staging_dir = temporary.path().join("staging");
    std::fs::create_dir_all(&install_dir).expect("install directory should be created");
    std::fs::create_dir_all(&staging_dir).expect("staging directory should be created");
    write_pair(&install_dir, b"old-cli", b"old-worker", "v1.0.0");
    write_pair(&staging_dir, b"new-cli", b"new-worker", "v2.0.0");

    let rollback_dir =
        activate_pair(&staging_dir, &install_dir, "v1.0.0").expect("release pair should activate");
    assert_eq!(
        prepare_startup(&install_dir).expect("replacement startup should begin verification"),
        StartupActivation::Pending(rollback_dir.clone())
    );

    assert_eq!(
        prepare_startup(&install_dir).expect("next startup should recover failed verification"),
        StartupActivation::Recovered(rollback_dir)
    );
    assert_eq!(read(&install_dir, "vulcanum"), b"old-cli");
    assert_eq!(read(&install_dir, "vulcanum-server"), b"old-worker");
    assert_eq!(read(&install_dir, VERSION_FILE), b"v1.0.0");
}

#[test]
fn startup_error_rolls_back_pending_activation() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let install_dir = temporary.path().join("install");
    let staging_dir = temporary.path().join("staging");
    std::fs::create_dir_all(&install_dir).expect("install directory should be created");
    std::fs::create_dir_all(&staging_dir).expect("staging directory should be created");
    write_pair(&install_dir, b"old-cli", b"old-worker", "v1.0.0");
    write_pair(&staging_dir, b"new-cli", b"new-worker", "v2.0.0");

    let rollback_dir =
        activate_pair(&staging_dir, &install_dir, "v1.0.0").expect("release pair should activate");
    assert!(matches!(
        prepare_startup(&install_dir).expect("replacement startup should begin verification"),
        StartupActivation::Pending(_)
    ));

    assert_eq!(
        rollback_pending_activation(&install_dir).expect("startup failure should roll back"),
        Some(rollback_dir)
    );
    assert_eq!(read(&install_dir, "vulcanum"), b"old-cli");
    assert_eq!(read(&install_dir, "vulcanum-server"), b"old-worker");
    assert_eq!(read(&install_dir, VERSION_FILE), b"v1.0.0");
}

#[test]
fn restores_existing_pair_when_second_binary_activation_fails() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let install_dir = temporary.path().join("install");
    let staging_dir = temporary.path().join("staging");
    std::fs::create_dir_all(&install_dir).expect("install directory should be created");
    std::fs::create_dir_all(&staging_dir).expect("staging directory should be created");
    write_pair(&install_dir, b"old-cli", b"old-worker", "v1.0.0");
    write_pair(&staging_dir, b"new-cli", b"new-worker", "v2.0.0");
    let replacements = Cell::new(0_u8);

    let error = activate_pair_with(
        &staging_dir,
        &install_dir,
        "v1.0.0",
        |source, destination| {
            let call = replacements.get() + 1;
            replacements.set(call);
            if call == 2 {
                return Err(std::io::Error::other("injected activation failure"));
            }
            std::fs::rename(source, destination)
        },
    )
    .expect_err("partial activation should fail");

    assert!(error
        .to_string()
        .contains("restored the previous binary pair"));
    assert_eq!(read(&install_dir, "vulcanum"), b"old-cli");
    assert_eq!(read(&install_dir, "vulcanum-server"), b"old-worker");
    assert_eq!(read(&install_dir, VERSION_FILE), b"v1.0.0");
}
#[test]
fn recovers_pair_after_interruption_between_binary_replacements() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let install_dir = temporary.path().join("install");
    let staging_dir = temporary.path().join("staging");
    std::fs::create_dir_all(&install_dir).expect("install directory should be created");
    std::fs::create_dir_all(&staging_dir).expect("staging directory should be created");
    write_pair(&install_dir, b"old-cli", b"old-worker", "v1.0.0");
    write_pair(&staging_dir, b"new-cli", b"new-worker", "v2.0.0");
    let replacements = Cell::new(0_u8);

    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = activate_pair_with(
            &staging_dir,
            &install_dir,
            "v1.0.0",
            |source, destination| {
                let call = replacements.get() + 1;
                replacements.set(call);
                if call == 2 {
                    panic!("simulated process interruption");
                }
                std::fs::rename(source, destination)
            },
        );
    }));

    assert!(interrupted.is_err());
    assert_eq!(read(&install_dir, "vulcanum"), b"new-cli");
    let rollback_dir = recover_interrupted_activation(&install_dir)
        .expect("interrupted activation should recover")
        .expect("recovery should report its rollback directory");
    assert!(rollback_dir.starts_with(install_dir.join(".vulcanum-rollback")));
    assert_eq!(read(&install_dir, "vulcanum"), b"old-cli");
    assert_eq!(read(&install_dir, "vulcanum-server"), b"old-worker");
    assert_eq!(read(&install_dir, VERSION_FILE), b"v1.0.0");
    assert!(recover_interrupted_activation(&install_dir)
        .expect("second recovery should be a no-op")
        .is_none());
}

fn write_pair(directory: &std::path::Path, cli: &[u8], worker: &[u8], version: &str) {
    std::fs::write(directory.join("vulcanum"), cli).expect("CLI should be written");
    std::fs::write(directory.join("vulcanum-server"), worker).expect("worker should be written");
    std::fs::write(directory.join(VERSION_FILE), version).expect("version should be written");
}

fn read(directory: &std::path::Path, name: &str) -> Vec<u8> {
    std::fs::read(directory.join(name)).expect("file should be readable")
}
