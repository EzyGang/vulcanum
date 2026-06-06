use vulcanum_shared::runtime::types::FinishRunArtifact;

pub(crate) fn read_finish_artifact(path: &std::path::Path) -> Option<FinishRunArtifact> {
    let raw = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str::<FinishRunArtifact>(&raw) {
        Ok(artifact) => {
            tracing::info!(status = ?artifact.status, "parsed finish artifact");
            Some(artifact)
        }
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to parse finish artifact");
            None
        }
    }
}
