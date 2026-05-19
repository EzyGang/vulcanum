pub mod workers;

use sqlx::{Executor, Postgres};

use crate::services::workers::errors::WorkersError;

pub trait Queryer<'c>: Executor<'c, Database = Postgres> {}

impl<'c> Queryer<'c> for &sqlx::PgPool {}

impl<'c> Queryer<'c> for &'c mut sqlx::PgConnection {}

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
