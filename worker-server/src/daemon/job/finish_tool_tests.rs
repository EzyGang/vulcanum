use crate::daemon::job::finish_tool::FINISH_RUN_TOOL_TS;

#[test]
fn finish_tool_returns_plain_string_result() {
    assert!(FINISH_RUN_TOOL_TS.contains("return `finish artifact written to ${path}`"));
    assert!(!FINISH_RUN_TOOL_TS.contains("return { ok: true }"));
}

#[test]
fn finish_tool_has_safe_artifact_path_fallback() {
    assert!(FINISH_RUN_TOOL_TS.contains("process.env.FINISH_ARTIFACT_PATH"));
    assert!(FINISH_RUN_TOOL_TS.contains("process.env.HOME"));
    assert!(FINISH_RUN_TOOL_TS.contains("process.cwd()"));
    assert!(FINISH_RUN_TOOL_TS.contains("mkdirSync(dirname(path), { recursive: true })"));
}

#[test]
fn finish_tool_uses_only_multi_pr_urls() {
    assert!(FINISH_RUN_TOOL_TS.contains("pr_urls: tool.schema.array"));
    assert!(!FINISH_RUN_TOOL_TS.contains("pr_url: tool.schema"));
    assert!(!FINISH_RUN_TOOL_TS.contains("args.pr_url)"));
    assert!(!FINISH_RUN_TOOL_TS.contains("args.pr_url,"));
}

#[test]
fn finish_tool_omits_unused_blocked_and_column_fields() {
    assert!(!FINISH_RUN_TOOL_TS.contains("blocked_reason"));
    assert!(!FINISH_RUN_TOOL_TS.contains("next_column"));
}
