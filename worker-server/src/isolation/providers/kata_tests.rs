use vulcanum_shared::runtime::types::ResourceLimits;

use crate::isolation::providers::kata::KataIsolation;

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

#[test]
fn resource_limits_default_vcpu() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.vcpu_count, 2);
}

#[test]
fn resource_limits_default_memory() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.memory_mib, 1_024);
}
