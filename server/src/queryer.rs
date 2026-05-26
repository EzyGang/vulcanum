use sqlx::{Executor, Postgres};

pub trait Queryer<'c>: Executor<'c, Database = Postgres> {}

impl<'c> Queryer<'c> for &sqlx::PgPool {}

impl<'c> Queryer<'c> for &'c mut sqlx::PgConnection {}
