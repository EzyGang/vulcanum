use super::support::{FakeServer, Fixture, Response, STATUS};

#[tokio::test]
async fn mode_specific_and_empty_inputs_fail_without_saving() {
    for (single_user, password_stdin, auth_code, no_browser) in [
        (true, false, Some("code".to_owned()), false),
        (true, false, None, true),
        (false, true, Some("code".to_owned()), false),
        (false, false, Some("   ".to_owned()), false),
    ] {
        let server = FakeServer::start(vec![
            Response::ok("GET", "/api/v1/status", STATUS),
            Response::ok(
                "GET",
                "/api/v1/auth/mode",
                format!(r#"{{"is_single_user":{single_user}}}"#),
            ),
        ]);
        let mut fixture = Fixture::new(true);
        let result = fixture
            .run(
                Some(server.url.clone()),
                password_stdin,
                auth_code,
                no_browser,
            )
            .await;

        server.finish();
        assert!(result.is_err());
        assert!(fixture.saved.is_empty());
    }
}
