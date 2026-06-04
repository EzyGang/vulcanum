pub fn continuation_prompt(turn: i32, max_turns: i32) -> String {
    format!(
        "[Continuation turn {turn}/{max_turns}]\n\
         The previous turn completed. The task remains active. \
         Continue from the current workspace state. Do not restart. \
         Focus on remaining work. When done, call the finish_run tool."
    )
}
