use crate::routes::auth::append_code_to_return_path;

#[test]
fn appends_callback_code_to_return_path() {
    assert_eq!(
        append_code_to_return_path("/login", "abc"),
        "/login?code=abc"
    );
    assert_eq!(
        append_code_to_return_path("/invites/token?source=oauth", "abc"),
        "/invites/token?source=oauth&code=abc"
    );
}
