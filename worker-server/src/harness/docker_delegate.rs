#[macro_export]
macro_rules! docker_isolation_delegate {
    ($name:ident, $runtime:literal) => {
        pub struct $name {
            pub(crate) inner: $crate::harness::container::DockerIsolation,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    inner: $crate::harness::container::DockerIsolation::new($runtime),
                }
            }

            #[allow(dead_code)]
            pub fn with_image(image: String) -> Self {
                Self {
                    inner: $crate::harness::container::DockerIsolation::with_image(image, $runtime),
                }
            }
        }

        impl std::default::Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl vulcanum_shared::runtime::isolation::IsolationProvider for $name {
            async fn prepare(
                &self,
                workdir: &std::path::Path,
                secrets: &std::collections::HashMap<String, String>,
                env_vars: &std::collections::HashMap<String, String>,
                limits: &vulcanum_shared::runtime::types::ResourceLimits,
                agents_md: &str,
                opencode_config: &str,
                repo_url: &str,
            ) -> Result<
                vulcanum_shared::runtime::types::IsolatedEnvironment,
                vulcanum_shared::runtime::errors::HarnessError,
            > {
                self.inner
                    .prepare(
                        workdir,
                        secrets,
                        env_vars,
                        limits,
                        agents_md,
                        opencode_config,
                        repo_url,
                    )
                    .await
            }

            async fn cleanup(&self, env: &vulcanum_shared::runtime::types::IsolatedEnvironment) {
                self.inner.cleanup(env).await;
            }
        }
    };
}
