use leptos::{server, ServerFnError};
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::person::db::{DbPerson, NewDbPerson};
use crate::person::Person;
use crate::role::RoleId;
use crate::schema::Schema;

#[derive(Debug, Serialize, Deserialize)]
pub enum Credentials {
    OAuth,
    Password { hash: String, salt: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub person: Person,
    pub credentials: Credentials,
    pub roles: Vec<RoleId>,
}

impl Schema for User {
    const TABLE: &'static str = "person";
    const SELECT: &'static str = "*, ->has_role->role as roles";
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct DbUser {
    #[serde(flatten)]
    pub person: DbPerson,
    pub credentials: Credentials,
    pub roles: Vec<surrealdb::sql::Thing>,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<DbUser> for User {
    fn from(item: DbUser) -> Self {
        Self {
            person: item.person.into(),
            credentials: item.credentials,
            roles: item.roles.into_iter().map(|r| r.into()).collect(),
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

#[server(ListUsers, "/api", "Url", "list_users")]
pub async fn list_users() -> Result<Vec<User>, ServerFnError> { backend::list_users().await }

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use crate::AppState;
    use leptos::use_context;

    pub async fn list_users() -> Result<Vec<User>, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(ServerFnError::new("No server state"))?;

        let people: Vec<DbUser> = app_state
            .db
            .query(format!("SELECT {} FROM {};", User::SELECT, User::TABLE))
            .await
            .map_err(|_| ServerFnError::new("db query failed"))?
            .take(0)?;

        let users: Vec<User> = people.into_iter().map(|x| x.into()).collect();
        Ok(users)
    }
}

