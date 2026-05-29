pub mod providers;

#[derive(Clone, Default)]
pub struct IntegrationProvidersRepository {}

impl IntegrationProvidersRepository {
    pub fn new() -> Self {
        Self {}
    }
}
