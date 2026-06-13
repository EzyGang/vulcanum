use crate::services::auth::service::github_oauth::validate_return_to;

#[test]
fn validates_safe_oauth_return_paths() {
    assert_eq!(
        validate_return_to(Some("/invites/token")),
        Some("/invites/token")
    );
    assert_eq!(
        validate_return_to(Some("/invites/token?source=login")),
        Some("/invites/token?source=login")
    );
}

#[test]
fn rejects_unsafe_oauth_return_paths() {
    assert_eq!(validate_return_to(None), None);
    assert_eq!(
        validate_return_to(Some("https://evil.test/invites/token")),
        None
    );
    assert_eq!(validate_return_to(Some("//evil.test/invites/token")), None);
    assert_eq!(validate_return_to(Some("invites/token")), None);
    assert_eq!(validate_return_to(Some("/../api/some-endpoint")), None);
    assert_eq!(validate_return_to(Some("/invites/../settings")), None);
    assert_eq!(
        validate_return_to(Some("/invites/token?next=..")),
        Some("/invites/token?next=..")
    );
    assert_eq!(validate_return_to(Some("/\\evil.test")), None);
    assert_eq!(validate_return_to(Some("/invites/token#fragment")), None);
    assert_eq!(
        validate_return_to(Some("/invites/token\nLocation: //evil.test")),
        None
    );
}
