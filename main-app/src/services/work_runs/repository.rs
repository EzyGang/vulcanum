pub mod work_runs;

#[derive(Clone)]
pub struct WorkRunsRepository {}

impl WorkRunsRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod work_runs_tests;
