use chrono::Duration;
use chrono::{DateTime, Utc};
use leptos::ServerFnError::{self, ServerError};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use surrealdb::sql::Thing;
use surrealdb::{engine::any::Any, Surreal};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDbSession {
    pub expires_at: DateTime<Utc>,
    pub user: Thing,
}

pub enum Fail {
    DbError(surrealdb::Error),
    NotCreated,
}

impl From<Fail> for ServerFnError {
    fn from(fail: Fail) -> Self {
        let msg = match fail {
            Fail::DbError(e) => format!("database error: {:?}", e),
            Fail::NotCreated => format!("session not created"),
        };
        ServerError(msg)
    }
}

pub async fn create_session(db: Surreal<Any>, person_id: String) -> Result<String, Fail> {
    let session_record: crate::db::Record = db
        .create("session")
        .content(NewDbSession {
            expires_at: chrono::Utc::now().add(Duration::days(1)),
            user: Thing {
                tb: "person".into(),
                id: person_id.into(),
            },
        })
        .await
        .map_err(|e| Fail::DbError(e))?
        .pop()
        .ok_or(Fail::NotCreated)?;

    Ok(session_record.id.id.to_string())
}

