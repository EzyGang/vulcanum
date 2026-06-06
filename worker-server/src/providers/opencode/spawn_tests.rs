use crate::providers::opencode::spawn::HOST_ENV_ALLOWLIST;

#[test]
fn host_env_allowlist_contains_expected_keys() {
    let expected = &["PATH", "TMPDIR", "HOME", "LANG"];
    for key in expected {
        assert!(
            HOST_ENV_ALLOWLIST.contains(key),
            "allowlist must contain {key}"
        );
    }
}

#[test]
fn host_env_allowlist_does_not_contain_sensitive_keys() {
    let sensitive = &[
        "GITHUB_TOKEN",
        "AWS_SECRET_ACCESS_KEY",
        "OPENAI_API_KEY",
        "KANEO_API_KEY",
    ];
    for key in sensitive {
        assert!(
            !HOST_ENV_ALLOWLIST.contains(key),
            "allowlist must not contain {key}"
        );
    }
}
