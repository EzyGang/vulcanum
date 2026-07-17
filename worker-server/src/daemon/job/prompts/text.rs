use vulcanum_shared::api::wire::WorkRunType;

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
        "The review identified CRITICAL or WARNINGS findings that must be resolved before the run can finish.\n\n\
Posted review:\n{review_body}\n\n\
Fix phase for the existing pull request:\n\
1. Stay on the current pull request branch. Do not create another branch or pull request.\n\
2. Re-read the applicable repository instructions, inspect the current diff, and resolve every CRITICAL and WARNINGS finding. Do not expand scope to SUGGESTIONS unless a required fix depends on one.\n\
3. Reproduce validation dependencies inside the container whenever feasible. Install and start local services such as PostgreSQL or Redis; report infrastructure as blocked only when it cannot be reproduced in or reached from the container after reasonable setup.\n\
4. Run the formatter, validation, and tests required for the changed repository. Review the resulting diff for regressions.\n\
5. Commit and push the fixes to the current pull request branch.\n\n\
Do not post a GitHub review during this fix phase and do not call finish_run. Stop after the fixes are pushed so the next turn can review the updated pull request."
    )
}

#[must_use]
pub fn review_after_fix_prompt(completed_fix_passes: i32, max_fix_passes: i32) -> String {
    format!(
        "[Review follow-up {completed_fix_passes}/{max_fix_passes}]\n\
Review the updated pull request from its base through the current PR head commit. Remain read-only: do not edit files, \
commit, push, or create a pull request. Verify that every previous CRITICAL and WARNINGS finding is resolved, inspect \
the complete diff for regressions, and run the applicable validation commands. Reproduce local validation services \
inside the container whenever feasible rather than treating them as blockers.\n\n\
Post exactly one new comment-only GitHub pull request review for the current PR head commit, using the required CRITICAL, \
WARNINGS, and SUGGESTIONS sections. Then call finish_run with status completed, review_url if available, and review_body."
    )
}

fn finish_run_instruction(work_type: WorkRunType) -> &'static str {
    match work_type {
        WorkRunType::Implementation => {
            "\n\nBefore ending the run, call the `finish_run` tool exactly once. \
Use `completed` when the requested work is done, `blocked` when external input is needed, \
or `failed` when the task cannot be completed. Install missing project dependencies, tools, runtimes, and local services \
needed to run formatter, validation, and test commands. Treat infrastructure as blocked only when it cannot be reproduced \
in or reached from the container after reasonable setup. Use `completed` only after running those commands for every \
repository you changed, or after recording the irreproducible dependency and attempted setup. \
If pull requests were created, include their URLs in `pr_urls`."
        }
        WorkRunType::PullRequestReview => {
            "\n\nBefore ending the review run, call the `finish_run` tool exactly once. \
Use `completed` when the review was posted or already existed, `blocked` when external input is needed, \
or `failed` when the review cannot be completed. Put posted review details in `review_url` and `review_body`; \
set `review_already_exists` only if a suitable review already exists for the current PR head commit. \
If the PR has new commits after the existing review, post a new review and leave `review_already_exists` false. \
The review_body must contain CRITICAL, WARNINGS, and SUGGESTIONS sections, and WARNINGS must call out missing \
or failing formatter, validation, or test commands. Install missing project dependencies, tools, runtimes, and local \
services needed to run those commands. Treat infrastructure as blocked only when it cannot be reproduced in or reached \
from the container after reasonable setup."
        }
    }
}
