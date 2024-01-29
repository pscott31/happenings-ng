pub mod auth;
pub mod booking;
pub mod config;
pub mod error_handling;
pub mod event;
pub mod generic_id;
pub mod person;
pub mod role;
pub mod schema;
pub mod square_api;
pub mod ticket;
pub mod user;

cfg_if::cfg_if! {
if #[cfg(not(target_arch = "wasm32"))] {
    pub mod axum;
    pub mod surreal;
    use surrealdb::{engine::any::Any, Surreal};
    use config::Config;
}}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Surreal<Any>,
}

