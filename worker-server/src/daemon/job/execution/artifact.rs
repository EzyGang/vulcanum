use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};

pub(crate) fn read_finish_artifact(path: &std::path::Path) -> Option<FinishRunArtifact> {
    let raw = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str::<FinishRunArtifact>(&raw) {
        Ok(mut artifact) => {
            if matches!(artifact.status, FinishStatus::Blocked) {
                let Some(reason) = artifact
                    .blocked_reason
                    .as_deref()
                    .map(str::trim)
                    .filter(|reason| !reason.is_empty())
                else {
                    tracing::warn!(
                        path = %path.display(),
                        "blocked finish artifact is missing a reason"
                    );
                    return None;
                };
                artifact.blocked_reason = Some(reason.to_owned());
            }

            tracing::info!(status = ?artifact.status, "parsed finish artifact");
            Some(artifact)
        }
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to parse finish artifact");
            None
        }
    }
}
