pub mod work_runs;

#[derive(Clone)]
#[allow(dead_code)]
pub struct WorkRunsRepository {}

impl WorkRunsRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod work_runs_tests;
