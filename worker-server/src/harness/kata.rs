use crate::harness::container::ContainerHarness;

pub struct KataHarness {
    pub(crate) inner: ContainerHarness,
}

impl KataHarness {
    pub fn new() -> Self {
        Self {
            inner: ContainerHarness::new("kata-runtime"),
        }
    }

    #[allow(dead_code)]
    pub fn with_image(image: String) -> Self {
        Self {
            inner: ContainerHarness::with_image(image, "kata-runtime"),
        }
    }
}

impl Default for KataHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for KataHarness {
    type Target = ContainerHarness;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for KataHarness {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
