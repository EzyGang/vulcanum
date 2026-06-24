use vulcanum_shared::api_types::WorkRunType;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};

use crate::daemon::job::review::review_loop::ReviewLoopState;

#[test]
fn review_loop_continues_for_actionable_review_then_stops_at_cap() {
    let mut state = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);

    let prompt = state
        .prompt_after_artifact(&review_artifact(
            "## CRITICAL\n- None\n\n## WARNINGS\n- Missing validation\n\n## SUGGESTIONS\n- None",
        ))
        .expect("actionable review should start fix pass");
    assert!(prompt.contains("Switch to implementation mode"));

    let prompt = state
        .prompt_after_fix_turn()
        .expect("fix turn should continue into re-review");
    assert!(prompt.contains("[Review follow-up 1/1]"));

    let prompt = state.prompt_after_artifact(&review_artifact(
        "## CRITICAL\n- None\n\n## WARNINGS\n- Still missing validation\n\n## SUGGESTIONS\n- None",
    ));
    assert!(
        prompt.is_none(),
        "review loop should stop at configured cap"
    );
}

#[test]
fn review_loop_submits_clean_review_immediately() {
    let mut state = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);

    let prompt = state.prompt_after_artifact(&review_artifact(
        "## CRITICAL\n- None\n\n## WARNINGS\n- None\n\n## SUGGESTIONS\n- Rename helper",
    ));

    assert!(prompt.is_none());
}

#[test]
fn implementation_loop_uses_plain_max_turns() {
    let state = ReviewLoopState::new(WorkRunType::Implementation, 4);

    assert_eq!(state.effective_max_turns(), 4);
}

#[must_use]
fn review_artifact(review_body: &str) -> FinishRunArtifact {
    FinishRunArtifact {
        status: FinishStatus::Completed,
        pr_urls: Vec::new(),
        summary: None,
        review_url: Some("https://github.com/acme/app/pull/1#pullrequestreview-1".to_owned()),
        review_body: Some(review_body.to_owned()),
        review_already_exists: false,
    }
}
