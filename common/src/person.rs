use leptos::{server, ServerFnError};
use serde::{Deserialize, Serialize};

pub type PersonId = GenericId<Person>;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct Person {
    pub id: PersonId,
    pub given_name: String,
    pub family_name: String,
    pub picture: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
use surrealdb::sql::Thing;

use crate::generic_id::{GenericId, TableName};
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DbPerson {
    pub id: Thing,
    pub given_name: String,
    pub family_name: String,
    pub picture: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<DbPerson> for Person {
    fn from(item: DbPerson) -> Self {
        Self {
            id: item.id.into(),
            given_name: item.given_name,
            family_name: item.family_name,
            picture: item.picture,
            email: item.email,
            phone: item.phone,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NewDbPerson {
    pub given_name: String,
    pub family_name: String,
    pub picture: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

impl TableName for Person {
    const TABLE_NAME: &'static str = "person";
}
impl Person {
    pub fn full_name(&self) -> String { format!("{} {}", self.given_name, self.family_name) }
}

#[server(GetPerson, "/api", "Url", "get_person")]
pub async fn get_person(id: PersonId) -> Result<Person, ServerFnError> { backend::get(id).await }

#[server(GetLoggedInPerson, "/api", "Url", "get_logged_in_person")]
pub async fn get_logged_in_person() -> Result<Person, ServerFnError> {
    backend::get_logged_in().await
}

#[server(PersonExists, "/api", "Url", "person_exists")]
pub async fn person_exists(email: String) -> Result<bool, ServerFnError> {
    backend::person_exists(email).await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use crate::{axum::LoggedInUser, db, AppState};
    use leptos::use_context;

    pub async fn get(id: PersonId) -> Result<Person, leptos::ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(ServerFnError::new("No server state"))?;

        let person: DbPerson = app_state
            .db
            .select(&id)
            .await
            .map_err(|_| ServerFnError::new("db query failed"))?
            .ok_or(ServerFnError::new(format!("no person {} found", id)))?;

        Ok(person.into())
    }

    pub async fn get_logged_in() -> Result<Person, leptos::ServerFnError> {
        let u = use_context::<Option<LoggedInUser>>();
        match u {
            None => Err(ServerFnError::new("no current user in context")),
            Some(None) => Err(ServerFnError::new("no current user in context")),
            Some(Some(LoggedInUser(person))) => Ok(person),
        }
    }

    pub async fn person_exists(email: String) -> Result<bool, leptos::ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(ServerFnError::new("No server state"))?;

        let people: Vec<db::Record> = app_state
            .db
            .query("SELECT * FROM person where email=$email;")
            .bind(("email", &email))
            .await?
            .take(0)?;
        Ok(!people.is_empty())
    }
}

