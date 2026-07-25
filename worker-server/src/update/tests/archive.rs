use crate::update::archive::{download, verify_and_extract_with_limits, MAX_CHECKSUM_BYTES};
use crate::update::tests::support::{checksum, release_archive, TestServer};

#[tokio::test]
async fn rejects_download_larger_than_configured_limit() {
    let server = TestServer::start(1, |_| {
        std::collections::HashMap::from([("/large".to_owned(), b"oversized".to_vec())])
    })
    .await;
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let destination = temporary.path().join("asset");
    let client = reqwest::Client::new();

    let error = download(
        &client,
        &format!("{}/large", server.base_url),
        &destination,
        4,
    )
    .await
    .expect_err("oversized download should be rejected");

    assert!(error.to_string().contains("4-byte limit"));
    assert!(!destination.exists());
}

#[test]
fn rejects_oversized_checksum_before_loading_it() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let archive_path = temporary.path().join("release.tar.gz");
    let checksum_path = temporary.path().join("release.tar.gz.sha256");
    std::fs::write(&archive_path, b"archive").expect("archive should be written");
    std::fs::write(&checksum_path, vec![b'0'; MAX_CHECKSUM_BYTES as usize + 1])
        .expect("checksum should be written");

    let error =
        verify_and_extract_with_limits(&archive_path, &checksum_path, temporary.path(), 16, 32)
            .expect_err("oversized checksum should be rejected");

    assert!(error.to_string().contains("checksum file exceeds"));
}

#[test]
fn rejects_archive_entry_larger_than_binary_limit() {
    let archive = release_archive(b"large-cli", b"worker");
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let archive_path = temporary.path().join("release.tar.gz");
    let checksum_path = temporary.path().join("release.tar.gz.sha256");
    std::fs::write(&archive_path, &archive).expect("archive should be written");
    std::fs::write(&checksum_path, checksum(&archive)).expect("checksum should be written");

    let error =
        verify_and_extract_with_limits(&archive_path, &checksum_path, temporary.path(), 8, 16)
            .expect_err("oversized binary should be rejected");

    assert!(error.to_string().contains("binary exceeds"));
}
