use vulcanum_shared::api_types::WorkRunType;

use crate::daemon::job::prompts::{continuation_prompt, initial_prompt};

#[test]
fn implementation_initial_prompt_requires_finish_run() {
    let prompt = initial_prompt(WorkRunType::Implementation, "Workspace\n", "Do the work");

    assert!(prompt.contains("Workspace"));
    assert!(prompt.contains("Do the work"));
    assert!(prompt.contains("call the `finish_run` tool exactly once"));
    assert!(prompt.contains("`pr_url`"));
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
    assert!(!prompt.contains("`pr_url`"));
}

#[test]
fn continuation_prompt_mentions_final_turn() {
    let prompt = continuation_prompt(2, 3);

    assert!(prompt.contains("[Continuation turn 3/3]"));
    assert!(prompt.contains("final allowed turn"));
}
