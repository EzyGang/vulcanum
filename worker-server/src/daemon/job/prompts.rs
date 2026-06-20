use vulcanum_shared::api_types::WorkRunType;

#[must_use]
pub fn initial_prompt(work_type: WorkRunType, workspace_prefix: &str, task_prompt: &str) -> String {
    format!(
        "{workspace_prefix}{task_prompt}{}",
        finish_run_instruction(work_type)
    )
}

pub fn continuation_prompt(turn: i32, max_turns: i32) -> String {
    let next_turn = turn + 1;
    let final_turn_instruction = match next_turn >= max_turns {
        true => " This is the final allowed turn; before stopping, call the finish_run tool.",
        false => "",
    };

    format!(
        "[Continuation turn {next_turn}/{max_turns}]\n\
         The previous turn completed. The task remains active. \
         Continue from the current workspace state. Do not restart. \
         The workspace may contain multiple sibling repositories; run commands from the relevant repo directory. \
         Focus on remaining work. When done, call the finish_run tool.{final_turn_instruction}"
    )
}

fn finish_run_instruction(work_type: WorkRunType) -> &'static str {
    match work_type {
        WorkRunType::Implementation => {
            "\n\nBefore ending the run, call the `finish_run` tool exactly once. \
Use `completed` when the requested work is done, `blocked` when external input is needed, \
or `failed` when the task cannot be completed. If pull requests were created, include their URLs in `pr_urls`."
        }
        WorkRunType::PullRequestReview => {
            "\n\nBefore ending the review run, call the `finish_run` tool exactly once. \
Use `completed` when the review was posted or already existed, `blocked` when external input is needed, \
or `failed` when the review cannot be completed. Put posted review details in `review_url` and `review_body`; \
set `review_already_exists` when the required review marker was already present."
        }
    }
}
