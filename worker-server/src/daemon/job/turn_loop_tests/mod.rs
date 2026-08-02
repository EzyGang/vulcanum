use vulcanum_shared::api::wire::WorkRunType;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};

use crate::daemon::job::review::review_loop::ReviewLoopState;
use crate::daemon::job::turn_loop::continuation::{
    continuation_prompt_for_dispatch, select_startup_action,
};
use crate::daemon::job::turn_loop::{PendingTurn, RecoveryTurn, StartupAction, StartupTurn};

#[test]
fn continuation_dispatch_prompt_has_provider_visible_idempotency_key() {
    let job_id = uuid::Uuid::new_v4();

    let prompt = continuation_prompt_for_dispatch("fix findings", job_id, 2);

    assert_eq!(
        prompt,
        format!("fix findings\n\n[Vulcanum continuation id: {job_id}:2]")
    );
}

#[test]
fn opencode_recovery_processes_actionable_artifact_before_waiting() {
    let mut review_loop = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);

    let startup = continued_startup(select_startup_action(
        Some(&actionable_review_artifact()),
        &mut review_loop,
        1,
        None,
        None,
    ));

    assert_fix_startup(&startup);
}

#[test]
fn omp_recovery_processes_actionable_artifact_before_generic_continuation() {
    let mut review_loop = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);
    let recovery_turn = RecoveryTurn {
        prompt: "generic recovery continuation".to_owned(),
        turn: 2,
    };

    let startup = continued_startup(select_startup_action(
        Some(&actionable_review_artifact()),
        &mut review_loop,
        1,
        None,
        Some(recovery_turn),
    ));

    assert_fix_startup(&startup);
    assert_ne!(startup.prompt, "generic recovery continuation");
}

#[test]
fn staged_transition_replays_before_artifact_or_recovery_prompt() {
    let mut review_loop = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);
    let pending_turn = PendingTurn {
        prompt: "persisted transition".to_owned(),
        cleanup_finish_artifact: true,
    };
    let recovery_turn = RecoveryTurn {
        prompt: "generic recovery continuation".to_owned(),
        turn: 3,
    };

    let startup = continued_startup(select_startup_action(
        Some(&actionable_review_artifact()),
        &mut review_loop,
        2,
        Some(pending_turn),
        Some(recovery_turn),
    ));

    assert_eq!(startup.prompt, "persisted transition");
    assert_eq!(startup.turn, 2);
    assert!(startup.cleanup_finish_artifact);
    assert!(startup.staged);
}

#[test]
fn opencode_recovery_submits_clean_artifact_before_waiting() {
    assert_terminal_artifact_submitted(None);
}

#[test]
fn omp_recovery_submits_clean_artifact_before_continuing() {
    assert_terminal_artifact_submitted(Some(RecoveryTurn {
        prompt: "generic recovery continuation".to_owned(),
        turn: 2,
    }));
}

#[test]
fn recovery_submits_actionable_artifact_when_fix_passes_are_exhausted() {
    let mut review_loop = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);
    let first = select_startup_action(
        Some(&actionable_review_artifact()),
        &mut review_loop,
        1,
        None,
        None,
    );
    assert!(matches!(first, Some(StartupAction::Continue(_))));

    let action = select_startup_action(
        Some(&actionable_review_artifact()),
        &mut review_loop,
        3,
        None,
        None,
    );

    assert_eq!(action, Some(StartupAction::SubmitArtifact));
}

fn assert_terminal_artifact_submitted(recovery_turn: Option<RecoveryTurn>) {
    let mut review_loop = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);
    let clean_artifact = FinishRunArtifact {
        review_body: Some(
            "## CRITICAL\n- None\n\n## WARNINGS\n- None\n\n## SUGGESTIONS\n- None".to_owned(),
        ),
        ..actionable_review_artifact()
    };

    let action = select_startup_action(
        Some(&clean_artifact),
        &mut review_loop,
        1,
        None,
        recovery_turn,
    );

    assert_eq!(action, Some(StartupAction::SubmitArtifact));
}

fn continued_startup(action: Option<StartupAction>) -> StartupTurn {
    match action {
        Some(StartupAction::Continue(startup)) => startup,
        Some(StartupAction::SubmitArtifact) | None => {
            panic!("expected recovered continuation")
        }
    }
}

fn assert_fix_startup(startup: &StartupTurn) {
    assert!(startup
        .prompt
        .contains("Fix phase for the existing pull request"));
    assert_eq!(startup.turn, 2);
    assert!(startup.cleanup_finish_artifact);
    assert!(!startup.staged);
}

fn actionable_review_artifact() -> FinishRunArtifact {
    FinishRunArtifact {
        status: FinishStatus::Completed,
        pr_urls: Vec::new(),
        summary: None,
        blocked_reason: None,
        review_url: Some("https://github.com/acme/app/pull/1#pullrequestreview-1".to_owned()),
        review_body: Some(
            "## CRITICAL\n- None\n\n## WARNINGS\n- Missing validation\n\n## SUGGESTIONS\n- None"
                .to_owned(),
        ),
        review_already_exists: true,
    }
}
