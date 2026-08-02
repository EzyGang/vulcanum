pub(crate) mod app;
pub mod login;
#[cfg(not(target_os = "windows"))]
pub mod self_delete;
#[cfg(not(target_os = "windows"))]
pub mod setup;
pub(crate) mod skills;
#[cfg(test)]
mod skills_tests;
#[cfg(not(target_os = "windows"))]
pub(crate) mod worker_updates;
#[cfg(all(test, not(target_os = "windows")))]
mod worker_updates_tests;
