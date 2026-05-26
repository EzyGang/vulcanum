pub mod users;

#[derive(Clone)]
pub struct UsersRepository {}

impl UsersRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod users_tests;
