mod process;
#[cfg(test)]
mod process_tests;
mod runner_session;
pub mod runtime;
mod session;
#[cfg(test)]
mod session_tests;

pub(super) const VULCANUM_OMP_PROVIDER_ENV: &str = "VULCANUM_OMP_PROVIDER";
pub(super) const VULCANUM_OMP_MODEL_ENV: &str = "VULCANUM_OMP_MODEL";
pub(super) const VULCANUM_OMP_SMOL_ENV: &str = "VULCANUM_OMP_SMOL";
