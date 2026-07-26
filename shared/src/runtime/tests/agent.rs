use serde_json::json;

use crate::runtime::agent::value_contains_text;

#[test]
fn detects_exact_dispatched_prompt_in_opencode_messages() {
    let prompt = "[Review follow-up 1/1]\nReview the updated pull request";
    let messages = json!([
        {
            "info": { "role": "user" },
            "parts": [{ "type": "text", "text": prompt }]
        },
        {
            "info": { "role": "assistant" },
            "parts": [{ "type": "text", "text": "Working" }]
        }
    ]);

    assert!(value_contains_text(&messages, prompt));
    assert!(!value_contains_text(&messages, "[Review follow-up 1/1]"));
}

#[test]
fn detects_exact_dispatched_prompt_in_omp_session_history() {
    let prompt = "[Continuation turn 3/3]\nFinish the active task";
    let session = json!({
        "messages": [
            {
                "role": "user",
                "content": [{ "type": "text", "text": prompt }]
            },
            {
                "role": "assistant",
                "content": [{ "type": "text", "text": "Done" }]
            }
        ]
    });

    assert!(value_contains_text(&session, prompt));
}
