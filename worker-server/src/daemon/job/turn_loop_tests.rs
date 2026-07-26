use vulcanum_shared::api::wire::WorkRunType;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};

use crate::daemon::job::review::review_loop::ReviewLoopState;
use crate::daemon::job::turn_loop::continuation::select_startup_turn;
use crate::daemon::job::turn_loop::{PendingTurn, RecoveryTurn, StartupTurn};

#[test]
fn opencode_recovery_processes_actionable_artifact_before_waiting() {
    let mut review_loop = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);

    let startup = select_startup_turn(
        Some(&actionable_review_artifact()),
        &mut review_loop,
        1,
        None,
        None,
    )
    .expect("actionable artifact should start a fix turn");

    assert_fix_startup(&startup);
}

#[test]
fn omp_recovery_processes_actionable_artifact_before_generic_continuation() {
    let mut review_loop = ReviewLoopState::new(WorkRunType::PullRequestReview, 1);
    let recovery_turn = RecoveryTurn {
        prompt: "generic recovery continuation".to_owned(),
        turn: 2,
    };

    let startup = select_startup_turn(
        Some(&actionable_review_artifact()),
        &mut review_loop,
        1,
        None,
        Some(recovery_turn),
    )
    .expect("actionable artifact should start a fix turn");

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

    let startup = select_startup_turn(
        Some(&actionable_review_artifact()),
        &mut review_loop,
        2,
        Some(pending_turn),
        Some(recovery_turn),
    )
    .expect("pending transition should replay");

    assert_eq!(startup.prompt, "persisted transition");
    assert_eq!(startup.turn, 2);
    assert!(startup.cleanup_finish_artifact);
    assert!(startup.staged);
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
        review_url: Some("https://github.com/acme/app/pull/1#pullrequestreview-1".to_owned()),
        review_body: Some(
            "## CRITICAL\n- None\n\n## WARNINGS\n- Missing validation\n\n## SUGGESTIONS\n- None"
                .to_owned(),
        ),
        review_already_exists: true,
    }
}
