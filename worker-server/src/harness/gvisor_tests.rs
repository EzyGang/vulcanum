use crate::harness::gvisor::GvisorHarness;
use crate::harness::{AgentHarness, ResourceLimits};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::test]
async fn test_gvisor_harness_uses_runsc_runtime() {
    let harness = GvisorHarness::with_image("hello-world".to_owned());
    let workdir = PathBuf::from("/tmp/vulcanum-test-gvisor");
    let secrets = HashMap::new();
    let limits = ResourceLimits::default();

    let result = harness
        .spawn("test prompt", &workdir, &secrets, &limits, "", "")
        .await;

    assert!(result.is_err());
}
