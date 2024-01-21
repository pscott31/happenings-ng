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


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDbSession {
    pub expires_at: DateTime<Utc>,
    pub user: Thing,
}

pub enum Fail {
    NoAppState,
    DbError(surrealdb::Error),
    NotCreated,
}

impl From<Fail> for ServerFnError {
    fn from(fail: Fail) -> Self {
        let msg = match fail {
            Fail::NoAppState => format!("no app state in context"),
            Fail::DbError(e) => format!("database error: {:?}", e),
            Fail::NotCreated => format!("session not created"),
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
        .map_err(|e| Fail::DbError(e))?
        .pop()
        .ok_or(Fail::NotCreated)?;

    Ok(session_record.id.id.to_string())
}

    }}

