pub mod queries;

use crate::queryer::Queryer;
use crate::services::workers::errors::WorkersError;

fn map_sqlx_error(err: sqlx::Error) -> WorkersError {
    WorkersError::Database(err)
}

#[derive(Clone)]
pub struct WorkersRepository {}

impl Default for WorkersRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkersRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod workers_tests;
