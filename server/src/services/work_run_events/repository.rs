pub mod work_run_events;

#[derive(Clone)]
pub struct WorkRunEventsRepository {}

impl Default for WorkRunEventsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkRunEventsRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod work_run_events_tests;
