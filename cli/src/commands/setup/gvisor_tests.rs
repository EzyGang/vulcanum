#[test]
fn test_gvisor_release_base_is_correct() {
    let base = super::gvisor::GVISOR_RELEASE_BASE;

    assert!(base.starts_with("https://storage.googleapis.com/gvisor/releases/"));
    assert!(base.contains("release/latest"));
}
