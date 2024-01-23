cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

use chrono::Duration;
use chrono::{DateTime, Utc};
use leptos::use_context;
use leptos::ServerFnError::{self, ServerError};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use surrealdb::sql::Thing;
use crate::AppState;
use crate::person::PersonId;
use crate::generic_id::{TableName, GenericId};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDbSession {
    pub expires_at: DateTime<Utc>,
    pub user: Thing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbSession {
    pub id: Thing,
    pub expires_at: DateTime<Utc>,
    pub user: Thing,
}

pub type SessionId = GenericId<Session>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub expires_at: DateTime<Utc>,
    pub user: PersonId,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<DbSession> for Session {
    fn from(item: DbSession) -> Self {
        Self {
            id: item.id.into(),
            expires_at: item.expires_at,
            user: item.user.into(),
        }
    }
}

impl TableName for Session {
    const TABLE_NAME: &'static str = "session";
}

pub enum Fail {
    NoAppState,
    DbError(surrealdb::Error),
    NotCreated,
}

impl From<Fail> for ServerFnError {
    fn from(fail: Fail) -> Self {
        let msg = match fail {
            Fail::NoAppState => "no app state in context".to_string(),
            Fail::DbError(e) => format!("database error: {:?}", e),
            Fail::NotCreated => "session not created".to_string(),
        };
        ServerError(msg)
    }
}

pub async fn create_session(person_id: PersonId) -> Result<String, Fail> {
    let app_state = use_context::<AppState>().ok_or(Fail::NoAppState)?;

    let session_record: crate::db::Record = app_state
        .db
        .create("session")
        .content(NewDbSession {
            expires_at: chrono::Utc::now().add(Duration::days(1)),
            user: person_id.into(),
        })
        .await
        .map_err(Fail::DbError)?
        .pop()
        .ok_or(Fail::NotCreated)?;

    Ok(session_record.id.id.to_string())
}

}}

