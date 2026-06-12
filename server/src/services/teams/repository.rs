pub mod queries;

#[derive(Clone, Default)]
pub struct TeamsRepository;

impl TeamsRepository {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}
