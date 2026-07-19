use super::fixtures::{board_responses, ignore_save, runtime, session, PROJECT_ID};
use crate::commands::app::board::tasks::{self, SearchOptions};
use crate::commands::app::board::{column, view};
use crate::tests::http::FakeServer;

#[tokio::test]
async fn board_view_limits_each_column_and_shows_identifiers_and_labels() {
    let server = FakeServer::start(board_responses());
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);

    view(PROJECT_ID, 1, None, &mut app)
        .await
        .expect("board view should succeed");
    server.finish();
    let output = String::from_utf8(output).expect("output should be utf8");

    for value in [
        "Automation workflow",
        "AUTOMATION",
        "enabled",
        "PICKUP",
        "To Do (to-do)",
        "IN REVIEW",
        "— (unset)",
        "DONE",
        "done (missing)",
        "COLUMN SLUG",
        "PROVIDER ID",
        "KAN-42",
        "task-1",
        "Fix parser",
        "backend",
        "In Progress",
        "1 more",
    ] {
        assert!(output.contains(value), "missing {value} in {output}");
    }
    assert!(!output.contains("Write docs"));
}

#[tokio::test]
async fn column_listing_accepts_name_and_paginates_tasks() {
    let server = FakeServer::start(board_responses());
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);

    column(PROJECT_ID, "To Do", 2, 1, None, &mut app)
        .await
        .expect("column page should succeed");
    server.finish();
    let output = String::from_utf8(output).expect("output should be utf8");

    assert!(output.contains("2 tasks, page 2/2"));
    assert!(output.contains("KAN-43"));
    assert!(output.contains("Write docs"));
    assert!(!output.contains("Fix parser"));
}

#[tokio::test]
async fn task_get_resolves_case_insensitive_slug_and_renders_body() {
    let server = FakeServer::start(board_responses());
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);

    tasks::get(PROJECT_ID, "kan-42", None, &mut app)
        .await
        .expect("task get should succeed");
    server.finish();
    let output = String::from_utf8(output).expect("output should be utf8");

    for value in [
        "KAN-42",
        "task-1",
        "Fix parser",
        "To Do",
        "BODY",
        "first body",
    ] {
        assert!(output.contains(value), "missing {value} in {output}");
    }
}

#[tokio::test]
async fn task_search_filters_text_column_and_label() {
    let server = FakeServer::start(board_responses());
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);

    tasks::search(
        SearchOptions {
            project_id: PROJECT_ID,
            query: Some("ui".to_owned()),
            column: Some("in-progress".to_owned()),
            label: Some("frontend".to_owned()),
            page: 1,
            page_size: 20,
            team: None,
        },
        &mut app,
    )
    .await
    .expect("task search should succeed");
    server.finish();
    let output = String::from_utf8(output).expect("output should be utf8");

    assert!(output.contains("1 tasks, page 1/1"));
    assert!(output.contains("KAN-44"));
    assert!(output.contains("Ship UI"));
    assert!(!output.contains("Fix parser"));
}
