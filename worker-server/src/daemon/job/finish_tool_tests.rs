use vulcanum_shared::api_types::WorkRunType;

use crate::daemon::job::finish_tool::finish_run_tool_ts;

#[test]
fn finish_tool_returns_plain_string_result() {
    for tool in [implementation_tool(), review_tool()] {
        assert!(tool.contains("return `finish artifact written to ${path}`"));
        assert!(!tool.contains("return { ok: true }"));
    }
}

#[test]
fn finish_tool_has_safe_artifact_path_fallback() {
    for tool in [implementation_tool(), review_tool()] {
        assert!(tool.contains("process.env.FINISH_ARTIFACT_PATH"));
        assert!(tool.contains("process.env.HOME"));
        assert!(tool.contains("process.cwd()"));
        assert!(tool.contains("mkdirSync(dirname(path), { recursive: true })"));
    }
}

#[test]
fn implementation_finish_tool_uses_pr_url_fields_only() {
    let tool = implementation_tool();

    assert!(tool.contains("pr_urls: tool.schema.array"));
    assert!(tool.contains("pr_urls: stringArrayOrEmpty(args.pr_urls)"));
    assert!(tool.contains("formatter, validation"));
    assert!(tool.contains("review_url: undefined"));
    assert!(tool.contains("review_body: undefined"));
    assert!(tool.contains("review_already_exists: false"));
    assert!(!tool.contains("pr_url: tool.schema"));
    assert!(!tool.contains("review_url: tool.schema"));
    assert!(!tool.contains("review_body: tool.schema"));
    assert!(!tool.contains("review_already_exists: tool.schema"));
}

#[test]
fn review_finish_tool_uses_review_fields_only() {
    let tool = review_tool();

    assert!(tool.contains("review_url: tool.schema.string"));
    assert!(tool.contains("review_body: tool.schema.string"));
    assert!(tool.contains("review_already_exists: tool.schema.boolean"));
    assert!(tool.contains("missing or failing formatter"));
    assert!(tool.contains("pr_urls: []"));
    assert!(tool.contains("current PR head commit"));
    assert!(tool.contains("new commits after the existing review"));
    assert!(!tool.contains("pr_urls: tool.schema.array"));
    assert!(!tool.contains("args.pr_urls"));
}

#[test]
fn finish_tool_is_selected_by_work_type() {
    assert_eq!(
        finish_run_tool_ts(WorkRunType::Implementation),
        implementation_tool()
    );
    assert_eq!(
        finish_run_tool_ts(WorkRunType::PullRequestReview),
        review_tool()
    );
}

#[test]
fn finish_tool_omits_unused_blocked_and_column_fields() {
    for tool in [implementation_tool(), review_tool()] {
        assert!(!tool.contains("blocked_reason"));
        assert!(!tool.contains("next_column"));
    }
}

#[must_use]
fn implementation_tool() -> String {
    finish_run_tool_ts(WorkRunType::Implementation)
}

#[must_use]
fn review_tool() -> String {
    finish_run_tool_ts(WorkRunType::PullRequestReview)
}
