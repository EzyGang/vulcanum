use crate::models::teams::model::{DEFAULT_PROMPT_TEMPLATE, DEFAULT_REVIEW_PROMPT_TEMPLATE};

#[test]
fn default_prompt_checks_repo_instructions_and_setup() {
    assert!(DEFAULT_PROMPT_TEMPLATE.contains("repository instructions"));
    assert!(DEFAULT_PROMPT_TEMPLATE.contains("project manifests"));
    assert!(DEFAULT_PROMPT_TEMPLATE.contains("focused setup subagent"));
    assert!(DEFAULT_PROMPT_TEMPLATE.contains("AGENTS.md"));
}

#[test]
fn default_review_prompt_checks_repo_instructions_and_validation() {
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("repository instructions"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("project manifests"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("every AGENTS.md"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("changed directories"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("read-only subagents"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("must not edit files"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("formatted and validated"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("Keep the final response concise"));
}
