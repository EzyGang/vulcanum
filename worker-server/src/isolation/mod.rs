pub(crate) mod checkout;
pub mod factory;
pub(crate) mod github_credentials;
pub mod providers;
pub(crate) mod workspace;

#[cfg(test)]
mod checkout_tests;
#[cfg(test)]
mod github_credentials_tests;
#[cfg(test)]
mod workspace_tests;
