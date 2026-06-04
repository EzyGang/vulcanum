use crate::harness::gvisor::GvisorIsolation;

#[test]
fn gvisor_inner_image_default() {
    let isolation = GvisorIsolation::new("test-image:v1".to_owned());
    assert!(!isolation.inner.image.is_empty());
}

#[test]
fn gvisor_inner_image_custom() {
    let isolation = GvisorIsolation::with_image("my-registry/agent:v1".to_owned());
    assert_eq!(isolation.inner.image, "my-registry/agent:v1");
}
