pub mod queries;

#[derive(Clone)]
pub struct WorkRunsRepository {}

impl Default for WorkRunsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkRunsRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
#[path = "repository/tests/work_runs_tests.rs"]
mod work_runs_tests;
