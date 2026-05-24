#[test]
fn test_kata_manager_url_is_correct() {
    let url = super::kata::KATA_MANAGER_URL;

    assert!(url.ends_with("kata-manager.sh"));
    assert!(!url.contains("kata-manager/kata-manager.sh"));
}

#[test]
fn test_kata_manager_url_is_raw_github() {
    let url = super::kata::KATA_MANAGER_URL;

    assert!(url.starts_with("https://raw.githubusercontent.com/"));
}
