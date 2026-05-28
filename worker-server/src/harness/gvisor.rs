use crate::harness::container::ContainerHarness;

pub struct GvisorHarness {
    pub(crate) inner: ContainerHarness,
}

impl GvisorHarness {
    pub fn new() -> Self {
        Self {
            inner: ContainerHarness::new("runsc"),
        }
    }

    #[allow(dead_code)]
    pub fn with_image(image: String) -> Self {
        Self {
            inner: ContainerHarness::with_image(image, "runsc"),
        }
    }
}

impl Default for GvisorHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for GvisorHarness {
    type Target = ContainerHarness;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for GvisorHarness {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
