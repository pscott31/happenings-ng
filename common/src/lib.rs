pub mod auth;
pub mod booking;
pub mod config;
pub mod error_handling;
pub mod event;
pub mod generic_id;
pub mod person;
pub mod square_api;
pub mod ticket;
pub mod user;

cfg_if::cfg_if! {
if #[cfg(not(target_arch = "wasm32"))] {
    pub mod axum;
    pub mod db;
    // use leptos::use_context;
    // use leptos::ServerFnError::ServerError;
    use surrealdb::{engine::any::Any, Surreal};
    use config::Config;
}}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Surreal<Any>,
}

// #[leptos::server(HelloWorld, "/api", "Url", "hello")]
// pub async fn list_users() -> Result<Vec<common::User>, leptos::ServerFnError> {
//     let app_state = use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;

//     let people: Vec<db::Person> = app_state
//         .db
//         .query("SELECT * FROM person;")
//         .await
//         .map_err(|_| ServerError("db query failed".to_string()))?
//         .take(0)?;

//     let users = people
//         .into_iter()
//         .map(|x| common::User {
//             id: x.id.to_string(),
//             given_name: x.given_name,
//             family_name: x.family_name,
//             email: x.email,
//             phone: x.phone,
//         })
//         .collect();
//     return Ok(users);
// }

