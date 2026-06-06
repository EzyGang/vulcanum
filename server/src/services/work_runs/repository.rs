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
mod work_runs_tests;
