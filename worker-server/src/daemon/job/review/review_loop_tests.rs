use vulcanum_shared::api::wire::WorkRunType;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};

use crate::daemon::job::review::review_loop::{ReviewLoopCheckpoint, ReviewLoopState};

#[test]
fn review_loop_continues_for_each_configured_follow_up_pass() {
    let mut state = ReviewLoopState::new(WorkRunType::PullRequestReview, 3);

    for pass in 1..=3 {
        let prompt = state
            .prompt_after_artifact(&actionable_review_artifact())
            .expect("actionable review should start a configured fix pass");
        assert!(prompt.contains("Fix phase for the existing pull request"));
        assert_eq!(state.progress().fix_pass, pass);

        let prompt = state
            .prompt_after_fix_turn()
            .expect("fix turn should continue into re-review");
        assert!(prompt.contains(&format!("[Review follow-up {pass}/3]")));
    }

    let prompt = state.prompt_after_artifact(&actionable_review_artifact());
    assert!(
        prompt.is_none(),
        "review loop should stop after every configured follow-up pass"
    );
}

#[test]
fn review_loop_submits_clean_review_immediately() {
    let mut state = ReviewLoopState::new(WorkRunType::PullRequestReview, 3);

    let prompt = state.prompt_after_artifact(&review_artifact(
        "## CRITICAL\n- None\n\n## WARNINGS\n- None\n\n## SUGGESTIONS\n- Rename helper",
    ));

    assert!(prompt.is_none());
}

#[test]
fn existing_actionable_review_still_starts_fix_pass() {
    let mut state = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);
    let mut artifact = actionable_review_artifact();
    artifact.review_already_exists = true;

    let prompt = state.prompt_after_artifact(&artifact);

    assert!(prompt.is_some());
    assert_eq!(state.progress().fix_pass, 1);
}

#[test]
fn recovered_follow_up_preserves_completed_fix_passes() {
    let checkpoint = ReviewLoopCheckpoint {
        fix_pass: 1,
        fixing: false,
    };
    let mut state = ReviewLoopState::resume(WorkRunType::PullRequestReview, 1, checkpoint);

    let prompt = state.prompt_after_artifact(&actionable_review_artifact());

    assert!(prompt.is_none());
    assert_eq!(state.progress().fix_pass, 1);
}

#[test]
fn recovered_fix_turn_returns_to_follow_up_review() {
    let checkpoint = ReviewLoopCheckpoint {
        fix_pass: 1,
        fixing: true,
    };
    let mut state = ReviewLoopState::resume(WorkRunType::PullRequestReview, 3, checkpoint);

    let prompt = state
        .prompt_after_fix_turn()
        .expect("recovered fix turn should continue into re-review");

    assert!(prompt.contains("[Review follow-up 1/3]"));
    assert_eq!(state.progress().fix_pass, 1);
}

#[test]
fn implementation_loop_uses_plain_max_turns() {
    let state = ReviewLoopState::new(WorkRunType::Implementation, 4);

    assert_eq!(state.effective_max_turns(), 4);
}

#[test]
fn review_loop_treats_max_turns_as_follow_up_pass_cap() {
    let state = ReviewLoopState::new(WorkRunType::PullRequestReview, 3);

    assert_eq!(state.effective_max_turns(), 7);
    assert_eq!(state.progress().max_fix_passes, 3);
}

#[test]
fn review_loop_one_configured_turn_allows_one_fix_pass() {
    let mut state = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);

    let prompt = state.prompt_after_artifact(&actionable_review_artifact());

    assert!(prompt.is_some());
    assert_eq!(state.effective_max_turns(), 3);
    assert_eq!(state.progress().fix_pass, 1);
    assert_eq!(state.progress().max_fix_passes, 1);
}

#[must_use]
fn actionable_review_artifact() -> FinishRunArtifact {
    review_artifact(
        "## CRITICAL\n- None\n\n## WARNINGS\n- Missing validation\n\n## SUGGESTIONS\n- None",
    )
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
