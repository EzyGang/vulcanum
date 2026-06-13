use std::fs;
use std::path::PathBuf;

use crate::commands::self_delete::remove_directory_best_effort;

fn temp_dir_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "vulcanum-self-delete-{name}-{}",
        std::process::id()
    ))
}

#[test]
fn remove_directory_best_effort_deletes_tree() {
    let dir = temp_dir_path("delete");
    let nested = dir.join("nested");
    fs::create_dir_all(&nested).expect("should create temp tree");
    fs::write(nested.join("state.json"), b"{}" as &[u8]).expect("should write file");

    remove_directory_best_effort(&dir);

    assert!(!dir.exists());
}

#[test]
fn remove_directory_best_effort_ignores_missing_directory() {
    let dir = temp_dir_path("missing");

    remove_directory_best_effort(&dir);

    assert!(!dir.exists());
}
