use crate::harness::container::DockerIsolation;
use crate::harness::kata::KataIsolation;

#[test]
fn container_image_default() {
    let isolation = DockerIsolation::new(Some("test-runtime"), "test-image:v1".to_owned());
    assert!(!isolation.image.is_empty());
}

#[test]
fn docker_plain_no_runtime() {
    let isolation = DockerIsolation::new(None, "test-image:v1".to_owned());
    assert!(isolation.runtime.is_none());
    assert!(!isolation.image.is_empty());
}

#[test]
fn kata_inner_image_default() {
    let isolation = KataIsolation::new("test-image:v1".to_owned());
    assert!(!isolation.inner.image.is_empty());
}

#[test]
fn kata_inner_image_custom() {
    let isolation = KataIsolation::with_image("my-registry/agent:v1".to_owned());
    assert_eq!(isolation.inner.image, "my-registry/agent:v1");
}
