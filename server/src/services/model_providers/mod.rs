pub mod catalog;
pub mod crypto;
pub mod errors;
pub mod model;
pub mod renderer;
pub mod repository;
pub mod service;

#[cfg(test)]
mod catalog_tests;
#[cfg(test)]
mod renderer_tests;
#[cfg(test)]
mod service_chatgpt_oauth_selection_tests;
#[cfg(test)]
mod service_chatgpt_oauth_test_support;
#[cfg(test)]
mod service_chatgpt_oauth_tests;
#[cfg(test)]
mod service_tests;
