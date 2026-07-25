use crate::update::release::{is_newer, select_package, GithubRelease};

#[test]
fn compares_semantic_release_versions() {
    assert!(is_newer("v1.2.3", "v1.3.0").expect("versions should parse"));
    assert!(!is_newer("1.3.0", "v1.3.0").expect("versions should parse"));
    assert!(!is_newer("v2.0.0", "v1.9.9").expect("versions should parse"));
    assert!(is_newer("v2.0.0-alpha.1", "v2.0.0").expect("versions should parse"));
    assert!(is_newer("invalid", "v2.0.0").is_err());
}

#[test]
fn selects_matching_archive_and_checksum_as_one_package() {
    let release: GithubRelease = serde_json::from_value(serde_json::json!({
        "tag_name": "v2.0.0",
        "assets": [
            {
                "name": "vulcanum-aarch64-apple-darwin.tar.gz.sha256",
                "browser_download_url": "https://example.test/mac.sha256"
            },
            {
                "name": "vulcanum-x86_64-unknown-linux-gnu.tar.gz",
                "browser_download_url": "https://example.test/linux.tar.gz"
            },
            {
                "name": "vulcanum-x86_64-unknown-linux-gnu.tar.gz.sha256",
                "browser_download_url": "https://example.test/linux.sha256"
            }
        ]
    }))
    .expect("release should deserialize");

    let package = select_package(&release, "x86_64-unknown-linux-gnu")
        .expect("compatible package should be selected");
    assert_eq!(
        package.archive_name,
        "vulcanum-x86_64-unknown-linux-gnu.tar.gz"
    );
    assert_eq!(package.archive_url, "https://example.test/linux.tar.gz");
    assert_eq!(package.checksum_url, "https://example.test/linux.sha256");
}

#[test]
fn rejects_release_without_a_compatible_pair() {
    let release: GithubRelease = serde_json::from_value(serde_json::json!({
        "tag_name": "v2.0.0",
        "assets": [{
            "name": "vulcanum-x86_64-unknown-linux-gnu.tar.gz",
            "browser_download_url": "https://example.test/linux.tar.gz"
        }]
    }))
    .expect("release should deserialize");

    let error = select_package(&release, "x86_64-unknown-linux-gnu")
        .expect_err("missing checksum asset should be rejected");
    assert!(error.to_string().contains(".sha256"));
}
