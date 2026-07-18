use std::io::Cursor;

use super::fixtures::{
    board_responses, ignore_save, project_responses, runtime, session, task_response, PROJECT_ID,
    PROVIDER_ID,
};
use crate::commands::app::board::tasks::{self, CreateOptions, EditOptions};
use crate::tests::http::{FakeServer, Response};

#[tokio::test]
async fn task_create_reads_complete_multiline_body_from_stdin() {
    let target = format!("/api/v1/task-board/providers/{PROVIDER_ID}/projects/KAN/tasks");
    let mut responses = project_responses();
    responses.push(Response::ok(
        "POST",
        &target,
        task_response("New task", "created body", "to-do"),
    ));
    let server = FakeServer::start(responses);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);
    let mut stdin = Cursor::new("First line\nSecond line\n");

    tasks::create(
        CreateOptions {
            project_id: PROJECT_ID,
            title: "New task".to_owned(),
            body: None,
            body_stdin: true,
            status: Some("to-do".to_owned()),
            priority: Some("high".to_owned()),
            team: None,
        },
        &mut app,
        &mut stdin,
    )
    .await
    .expect("task create should succeed");
    let requests = server.finish();
    let create = requests
        .iter()
        .find(|request| request.method == "POST" && request.target == target)
        .expect("create request should be recorded");

    assert!(create
        .body
        .contains(r#""body":"First line\nSecond line\n""#));
    assert!(create.body.contains(r#""priority":"high""#));
    assert!(String::from_utf8(output)
        .expect("output should be utf8")
        .contains("Created task KAN-42 (task-1)"));
}

#[tokio::test]
async fn task_edit_resolves_slug_and_preserves_omitted_body() {
    let target = format!("/api/v1/task-board/providers/{PROVIDER_ID}/tasks/task-1");
    let mut responses = board_responses();
    responses.push(Response::ok(
        "PATCH",
        &target,
        task_response("Fixed parser", "first body", "to-do"),
    ));
    let server = FakeServer::start(responses);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);
    let mut stdin = Cursor::new(Vec::<u8>::new());

    tasks::edit(
        EditOptions {
            project_id: PROJECT_ID,
            task: "KAN-42".to_owned(),
            title: Some("Fixed parser".to_owned()),
            body: None,
            body_stdin: false,
            team: None,
        },
        &mut app,
        &mut stdin,
    )
    .await
    .expect("task edit should succeed");
    let requests = server.finish();
    let update = requests
        .iter()
        .find(|request| request.method == "PATCH" && request.target == target)
        .expect("update request should be recorded");

    assert!(update.body.contains(r#""title":"Fixed parser""#));
    assert!(update.body.contains(r#""body":"first body""#));
    assert!(String::from_utf8(output)
        .expect("output should be utf8")
        .contains("Updated task KAN-42 (task-1)"));
}

#[tokio::test]
async fn task_move_resolves_slug_and_column_name() {
    let target = format!("/api/v1/task-board/providers/{PROVIDER_ID}/tasks/task-1/status");
    let mut responses = board_responses();
    responses.push(Response::ok(
        "PATCH",
        &target,
        r#"{"task_id":"task-1","status":"in-progress"}"#,
    ));
    let server = FakeServer::start(responses);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);

    tasks::move_task(PROJECT_ID, "kan-42", "In Progress", None, &mut app)
        .await
        .expect("task move should succeed");
    let requests = server.finish();
    let movement = requests
        .iter()
        .find(|request| request.method == "PATCH" && request.target == target)
        .expect("move request should be recorded");

    assert_eq!(movement.body, r#"{"status":"in-progress"}"#);
    assert!(String::from_utf8(output)
        .expect("output should be utf8")
        .contains("Moved task KAN-42 (task-1) to In Progress"));
}
