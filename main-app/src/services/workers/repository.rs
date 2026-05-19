pub mod workers;

use crate::queryer::Queryer;
use crate::services::workers::errors::WorkersError;

fn map_sqlx_error(err: sqlx::Error) -> WorkersError {
    WorkersError::Database(err)
}

#[derive(Clone)]
pub struct WorkersRepository {}

impl WorkersRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod workers_tests;
