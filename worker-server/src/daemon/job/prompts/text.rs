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
Install missing project dependencies, tools, runtimes, or local services needed to validate the changed work. \
Focus on remaining work. When done, call the finish_run tool.{final_turn_instruction}"
    )
}

#[must_use]
pub fn review_fix_prompt(review_body: &str) -> String {
    format!(
        "The review found CRITICAL or WARNINGS items that must be fixed before this run can finish.\n\n\
Review body:\n{review_body}\n\n\
Switch to implementation mode for the existing pull request. Address only the CRITICAL and WARNINGS items, \
install missing project dependencies, tools, runtimes, or local services needed to validate the changed work, \
run the formatter, validation, and test commands required for the changed repository, commit and push the fixes \
to the current pull request branch, then stop. Do not create a new pull request and do not call finish_run after the fix turn."
    )
}

#[must_use]
pub fn review_after_fix_prompt(completed_fix_passes: i32, max_fix_passes: i32) -> String {
    format!(
        "[Review follow-up {completed_fix_passes}/{max_fix_passes}]\n\
Review the updated pull request after the pushed fixes. Follow the original review instructions and post a new \
comment-only GitHub pull request review for the current PR head commit. When done, call finish_run with status \
completed, review_url if available, and review_body."
    )
}

fn finish_run_instruction(work_type: WorkRunType) -> &'static str {
    match work_type {
        WorkRunType::Implementation => {
            "\n\nBefore ending the run, call the `finish_run` tool exactly once. \
Use `completed` when the requested work is done, `blocked` when external input is needed, \
or `failed` when the task cannot be completed. You may install missing project dependencies, tools, runtimes, or local services \
needed to run formatter, validation, and test commands. Use `completed` only after running those commands for every \
repository you changed, or after recording why a required setup or command could not run. \
If pull requests were created, include their URLs in `pr_urls`."
        }
        WorkRunType::PullRequestReview => {
            "\n\nBefore ending the review run, call the `finish_run` tool exactly once. \
Use `completed` when the review was posted or already existed, `blocked` when external input is needed, \
or `failed` when the review cannot be completed. Put posted review details in `review_url` and `review_body`; \
set `review_already_exists` only if a suitable review already exists for the current PR head commit. \
If the PR has new commits after the existing review, post a new review and leave `review_already_exists` false. \
The review_body must contain CRITICAL, WARNINGS, and SUGGESTIONS sections, and WARNINGS must call out missing \
or failing formatter, validation, or test commands. You may install missing project dependencies, tools, runtimes, or \
local services needed to run those commands before reporting them as blocked."
        }
    }
}
