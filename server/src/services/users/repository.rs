pub mod queries;

#[derive(Clone)]
pub struct UsersRepository {}

impl Default for UsersRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl UsersRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod users_tests;
