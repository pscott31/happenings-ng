use serde::{Deserialize, Serialize};

use crate::person::Person;
#[cfg(not(target_arch = "wasm32"))]
use crate::person::{DbPerson, NewDbPerson};

#[derive(Debug, Serialize, Deserialize)]
pub enum Credentials {
    OAuth,
    Password { hash: String, salt: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub person: Person,
    pub credentials: Credentials,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct DbUser {
    #[serde(flatten)]
    pub person: DbPerson,
    pub credentials: Credentials,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<DbUser> for User {
    fn from(item: DbUser) -> Self {
        Self {
            person: item.person.into(),
            credentials: item.credentials,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct NewDbUser {
    #[serde(flatten)]
    pub person: NewDbPerson,
    pub credentials: Credentials,
}

#[leptos::server(HelloWorld, "/api", "Url", "list_users")]
pub async fn list_users() -> Result<Vec<User>, leptos::ServerFnError> {
    backend::list_users().await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use crate::AppState;
    use leptos::{use_context, ServerFnError::ServerError};

    pub async fn list_users() -> Result<Vec<User>, leptos::ServerFnError> {
        let app_state =
            use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;

        let people: Vec<DbUser> = app_state
            .db
            .query("SELECT * FROM person;")
            .await
            .map_err(|_| ServerError("db query failed".to_string()))?
            .take(0)?;

        let users: Vec<User> = people.into_iter().map(|x| x.into()).collect();
        return Ok(users);
    }
}

