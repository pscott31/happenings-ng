use happenings_macro::{generate_db, generate_new};
use serde::{Deserialize, Serialize};

pub type PersonID = String;

#[generate_new]
#[generate_db]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct Person {
    pub id: PersonID,
    pub given_name: String,
    pub family_name: String,
    pub picture: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

impl Person {
    pub fn full_name(&self) -> String { format!("{} {}", self.given_name, self.family_name) }
}

#[leptos::server(GetEvent, "/api", "Url", "get_person")]
pub async fn get_person(id: String) -> Result<Person, leptos::ServerFnError> {
    backend::get(id).await
}

#[leptos::server(GetLoggedInPerson, "/api", "Url", "get_logged_in_person")]
pub async fn get_logged_in_person() -> Result<Person, leptos::ServerFnError> {
    backend::get_logged_in().await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use crate::AppState;
    use leptos::{use_context, ServerFnError::ServerError};

    pub async fn get(id: String) -> Result<Person, leptos::ServerFnError> {
        let app_state =
            use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;
        let thing =
            surrealdb::sql::thing(id.as_ref()).map_err(|_| ServerError("Bad id".to_string()))?;

        let mut res: Vec<Person> = app_state
            .db
            .query("SELECT type::string(id) as id, * from person where id=$req_id;")
            .bind(("req_id", thing))
            .await
            .map_err(|_| ServerError("db query failed".to_string()))?
            .take(0)?;

        let person = res
            .pop()
            .ok_or(ServerError(format!("no person {} found", id).to_string()))?;

        return Ok(person);
    }

    pub async fn get_logged_in() -> Result<Person, leptos::ServerFnError> {
        let logged_in_person =
            use_context::<Person>().ok_or(ServerError("No logged in person".to_string()))?;
        Ok(logged_in_person)
    }
}

