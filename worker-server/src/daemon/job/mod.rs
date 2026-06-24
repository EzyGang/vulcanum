#[path = "execution/artifact.rs"]
pub(crate) mod artifact;
#[path = "execution/event_reporter.rs"]
pub(crate) mod event_reporter;
#[path = "tool/finish_tool.rs"]
pub(crate) mod finish_tool;
pub(crate) mod orchestrate;
#[path = "prompts/prompts.rs"]
mod prompts;
#[path = "review/review_loop.rs"]
pub(crate) mod review_loop;
#[path = "execution/submit.rs"]
pub(crate) mod submit;
pub(crate) mod turn_loop;

pub(crate) use orchestrate::handle_job;

#[cfg(test)]
#[path = "execution/artifact_tests.rs"]
mod artifact_tests;
#[cfg(test)]
#[path = "tool/finish_tool_tests.rs"]
mod finish_tool_tests;
#[cfg(test)]
#[path = "prompts/prompts_tests.rs"]
mod prompts_tests;
#[cfg(test)]
#[path = "review/review_loop_tests.rs"]
mod review_loop_tests;
#[cfg(test)]
#[path = "execution/submit_tests.rs"]
mod submit_tests;
