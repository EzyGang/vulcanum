use super::{
    submit_failed_result, submit_turn_result, watch, FailedResult, FinishRunArtifact, FinishStatus,
    Path, RunningSession, SessionExport, TurnLoopCtx,
};

pub(super) async fn wait_for_session(
    running_session: &mut Box<dyn RunningSession>,
    cancel_rx: &mut watch::Receiver<bool>,
    turn: i32,
    ctx: &TurnLoopCtx,
) -> Option<SessionExport> {
    if *cancel_rx.borrow() {
        return cancel_running_session(running_session, turn, ctx).await;
    }

    loop {
        tokio::select! {
            result = running_session.wait() => {
                return match result {
                    Ok(export) => Some(export),
                    Err(error) => {
                        tracing::error!(
                            worker_id = %ctx.worker_id,
                            work_run_id = %ctx.job_id,
                            turn = turn,
                            error = %error,
                            "session wait failed",
                        );
                        let _ = running_session.cancel().await;
                        ctx.reporter
                            .emit(
                                "session.failed",
                                serde_json::json!({"reason": "wait_error", "turn": turn}),
                            )
                            .await;
                        submit_failed_result(
                            ctx.client.clone(),
                            ctx.worker_state.clone(),
                            ctx.journal.clone(),
                            ctx.job_id,
                            &FailedResult::empty(),
                        )
                        .await;
                        None
                    }
                };
            }
            changed = cancel_rx.changed() => match changed {
                Ok(()) if *cancel_rx.borrow() => {
                    return cancel_running_session(running_session, turn, ctx).await;
                }
                Ok(()) => (),
                Err(_) => (),
            },
        }
    }
}

async fn cancel_running_session(
    running_session: &mut Box<dyn RunningSession>,
    turn: i32,
    ctx: &TurnLoopCtx,
) -> Option<SessionExport> {
    tracing::warn!(
        worker_id = %ctx.worker_id,
        work_run_id = %ctx.job_id,
        turn = turn,
        "server-requested cancellation received, cancelling running session",
    );
    if let Err(error) = running_session.cancel().await {
        tracing::warn!(
            worker_id = %ctx.worker_id,
            work_run_id = %ctx.job_id,
            turn = turn,
            error = %error,
            "provider cancellation returned an error after cleanup",
        );
    }
    ctx.reporter
        .emit(
            "session.cancelled",
            serde_json::json!({"reason": "server_requested", "turn": turn}),
        )
        .await;
    match running_session.export().await {
        Ok(export) => {
            submit_turn_result(
                &ctx.client,
                &ctx.worker_state,
                &ctx.journal,
                ctx.job_id,
                &export,
                None,
            )
            .await;
        }
        Err(error) => {
            tracing::warn!(
                worker_id = %ctx.worker_id,
                work_run_id = %ctx.job_id,
                turn = turn,
                error = %error,
                "failed to export cancelled session",
            );
            submit_failed_result(
                ctx.client.clone(),
                ctx.worker_state.clone(),
                ctx.journal.clone(),
                ctx.job_id,
                &FailedResult::empty(),
            )
            .await;
        }
    }
    None
}

pub(super) async fn submit_provider_failure(
    ctx: &TurnLoopCtx,
    turn: i32,
    session_export: &SessionExport,
) {
    tracing::warn!(
        worker_id = %ctx.worker_id,
        work_run_id = %ctx.job_id,
        turn = turn,
        exit_code = session_export.exit_code,
        provider_error = ?session_export.failure_payload,
        "session failed, not continuing turn loop",
    );
    ctx.reporter
        .emit(
            "session.failed",
            serde_json::json!({
                "reason": "nonzero_exit",
                "turn": turn,
                "exit_code": session_export.exit_code,
                "tokens_used": session_export.tokens_used,
                "model_used": session_export.model_used.clone(),
                "provider_error": session_export.failure_payload.clone(),
            }),
        )
        .await;
    submit_turn_result(
        &ctx.client,
        &ctx.worker_state,
        &ctx.journal,
        ctx.job_id,
        session_export,
        None,
    )
    .await;
}

pub(super) fn remove_finish_artifact(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to remove finish artifact")
        }
    }
}

#[must_use]
pub(super) fn finish_exit_code(artifact: &FinishRunArtifact) -> i32 {
    match artifact.status {
        FinishStatus::Completed => 0,
        FinishStatus::Failed | FinishStatus::Blocked => 1,
    }
}
