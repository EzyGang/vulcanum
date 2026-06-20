use crate::daemon::job::submit::{submit_result_request, SubmitResultParams};

#[test]
fn submit_result_includes_review_fields() {
    let request = submit_result_request(SubmitResultParams {
        pr_urls: Vec::new(),
        exit_code: 0,
        tokens_used: 10,
        duration_ms: 100,
        input_tokens: 1,
        output_tokens: 2,
        cache_read_tokens: 3,
        cache_write_tokens: 4,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        review_url: Some("https://github.com/acme/widgets/pull/42#pullrequestreview-1".to_owned()),
        review_body: Some("Looks good".to_owned()),
        review_already_exists: true,
    });

    assert_eq!(
        request.review_url.as_deref(),
        Some("https://github.com/acme/widgets/pull/42#pullrequestreview-1")
    );
    assert_eq!(request.review_body.as_deref(), Some("Looks good"));
    assert!(request.review_already_exists);
}
