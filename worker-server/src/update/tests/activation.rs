use std::cell::Cell;

use crate::update::activation::{activate_pair, activate_pair_with};
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

fn write_pair(directory: &std::path::Path, cli: &[u8], worker: &[u8], version: &str) {
    std::fs::write(directory.join("vulcanum"), cli).expect("CLI should be written");
    std::fs::write(directory.join("vulcanum-server"), worker).expect("worker should be written");
    std::fs::write(directory.join(VERSION_FILE), version).expect("version should be written");
}

fn read(directory: &std::path::Path, name: &str) -> Vec<u8> {
    std::fs::read(directory.join(name)).expect("file should be readable")
}
