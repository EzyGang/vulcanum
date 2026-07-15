use vulcanum_shared::api_types::WorkRunType;

use crate::daemon::job::prompts::text::{
    continuation_prompt, initial_prompt, review_after_fix_prompt, review_fix_prompt,
};

#[test]
fn implementation_initial_prompt_requires_finish_run() {
    let prompt = initial_prompt(WorkRunType::Implementation, "Workspace\n", "Do the work");

    assert!(prompt.contains("Workspace"));
    assert!(prompt.contains("Do the work"));
    assert!(prompt.contains("call the `finish_run` tool exactly once"));
    assert!(prompt.contains("formatter, validation"));
    assert!(prompt.contains("every repository you changed"));
    assert!(prompt.contains("Install missing project dependencies"));
    assert!(prompt.contains("only when it cannot be reproduced"));
    assert!(prompt.contains("`pr_urls`"));
    assert!(!prompt.contains("`pr_url`"));
}

#[test]
fn review_initial_prompt_uses_review_artifact_fields() {
    let prompt = initial_prompt(
        WorkRunType::PullRequestReview,
        "Workspace\n",
        "Review the PR",
    );

    assert!(prompt.contains("review run"));
    assert!(prompt.contains("`review_url`"));
    assert!(prompt.contains("`review_body`"));
    assert!(prompt.contains("`review_already_exists`"));
    assert!(prompt.contains("current PR head commit"));
    assert!(prompt.contains("new commits after the existing review"));
    assert!(prompt.contains("CRITICAL"));
    assert!(prompt.contains("WARNINGS"));
    assert!(prompt.contains("SUGGESTIONS"));
    assert!(prompt.contains("missing"));
    assert!(prompt.contains("formatter, validation"));
    assert!(prompt.contains("Install missing project dependencies"));
    assert!(prompt.contains("only when it cannot be reproduced"));
    assert!(!prompt.contains("`pr_url`"));
}

#[test]
fn continuation_prompt_mentions_final_turn() {
    let prompt = continuation_prompt(2, 3);

    assert!(prompt.contains("[Continuation turn 3/3]"));
    assert!(prompt.contains("final allowed turn"));
    assert!(prompt.contains("Install missing project dependencies"));
    assert!(prompt.contains("local services"));
}

#[test]
fn review_fix_prompt_switches_to_existing_pr_fix_mode() {
    let prompt = review_fix_prompt("## WARNINGS\n- Missing validation");

    assert!(prompt.contains("Fix phase for the existing pull request"));
    assert!(prompt.contains("current pull request branch"));
    assert!(prompt.contains("every CRITICAL and WARNINGS finding"));
    assert!(prompt.contains("PostgreSQL or Redis"));
    assert!(prompt.contains("only when it cannot be reproduced"));
    assert!(prompt.contains("Do not create another branch or pull request"));
    assert!(prompt.contains("do not call finish_run"));
}

#[test]
fn review_after_fix_prompt_requires_new_review() {
    let prompt = review_after_fix_prompt(1, 2);

    assert!(prompt.contains("[Review follow-up 1/2]"));
    assert!(prompt.contains("updated pull request"));
    assert!(prompt.contains("current PR head commit"));
    assert!(prompt.contains("call finish_run"));
    assert!(prompt.contains("Remain read-only"));
    assert!(prompt.contains("every previous CRITICAL and WARNINGS finding"));
    assert!(prompt.contains("complete diff for regressions"));
    assert!(prompt.contains("required CRITICAL"));
}
