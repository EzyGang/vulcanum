use std::fs;

use uuid::Uuid;
use vulcanum_shared::runtime::types::FinishStatus;

use crate::daemon::job::execution::artifact::read_finish_artifact;

#[test]
fn parses_review_finish_artifact_fields() {
    let path = std::env::temp_dir().join(format!("finish-artifact-{}.json", Uuid::new_v4()));
    fs::write(
        &path,
        r#"{
            "status": "completed",
            "review_url": "https://github.com/acme/widgets/pull/42#pullrequestreview-1",
            "review_body": "Looks good",
            "review_already_exists": true
        }"#,
    )
    .expect("artifact should be written");

    let artifact = read_finish_artifact(&path).expect("artifact should parse");
    let _ = fs::remove_file(path);

    assert_eq!(artifact.status, FinishStatus::Completed);
    assert_eq!(
        artifact.review_url.as_deref(),
        Some("https://github.com/acme/widgets/pull/42#pullrequestreview-1")
    );
    assert_eq!(artifact.review_body.as_deref(), Some("Looks good"));
    assert!(artifact.review_already_exists);
}
