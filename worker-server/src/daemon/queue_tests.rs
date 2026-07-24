use uuid::Uuid;

use crate::daemon::queue::JobTracker;

#[tokio::test]
async fn tracker_rejects_duplicate_until_job_finishes() {
    let tracker = JobTracker::default();
    let job_id = Uuid::new_v4();

    assert!(tracker.reserve(job_id).await);
    assert!(!tracker.reserve(job_id).await);

    tracker.release(job_id).await;

    assert!(tracker.reserve(job_id).await);
}
