use crate::services::teams::model::DEFAULT_REVIEW_PROMPT_TEMPLATE;

#[test]
fn default_review_prompt_checks_repo_instructions_and_validation() {
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("every AGENTS.md"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("changed directories"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("formatted and validated"));
    assert!(DEFAULT_REVIEW_PROMPT_TEMPLATE.contains("formatter, validation, or test commands"));
}
