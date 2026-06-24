pub mod auth;
pub mod catalog;
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
mod service_tests;
