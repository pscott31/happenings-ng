use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::person::{DbPerson, NewDbPerson};

#[derive(Debug, Serialize, Deserialize)]
pub enum Credentials {
    OAuth,
    Password { hash: String, salt: String },
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct DbUser {
    #[serde(flatten)]
    pub person: DbPerson,
    pub credentials: Credentials,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct NewDbUser {
    #[serde(flatten)]
    pub person: NewDbPerson,
    pub credentials: Credentials,
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

