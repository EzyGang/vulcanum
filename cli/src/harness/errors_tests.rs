use crate::harness::errors::HarnessError;

#[test]
fn display_install() {
    let e = HarnessError::Install("missing binary".to_owned());
    assert_eq!(e.to_string(), "install error: missing binary");
}

#[test]
fn display_container_boot() {
    let e = HarnessError::ContainerBoot("bad config".to_owned());
    assert_eq!(e.to_string(), "container boot error: bad config");
}

#[test]
fn display_timeout() {
    let e = HarnessError::Timeout(1_800);
    assert_eq!(e.to_string(), "job timed out after 1800s");
}

#[test]
fn display_opencode_crash() {
    let e = HarnessError::OpenCodeCrash("segfault".to_owned());
    assert_eq!(e.to_string(), "opencode crashed: segfault");
}

#[test]
fn display_output_parse() {
    let e = HarnessError::OutputParse("no pr url".to_owned());
    assert_eq!(e.to_string(), "output parse error: no pr url");
}
