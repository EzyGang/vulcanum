use crate::runtime::types::ResourceLimits;

#[test]
fn resource_limits_default() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.max_duration_secs, 1_800);
    assert_eq!(limits.vcpu_count, 2);
    assert_eq!(limits.memory_mib, 1_024);
}
