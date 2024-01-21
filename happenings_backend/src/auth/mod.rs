pub mod oauth;
pub mod password;

use crate::error_handling::AppError;
use anyhow::anyhow;
use chrono::Duration;
use happenings::db::*;
use std::ops::Add;
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

async fn create_session(db: Surreal<Any>, person_id: String) -> Result<String, AppError> {
    let session_record: Record = db
        .create("session")
        .content(Session {
            expires_at: chrono::Utc::now().add(Duration::days(1)),
            user: Thing {
                tb: "person".into(),
                id: person_id.into(),
            },
        })
        .await?
        .pop()
        .ok_or(anyhow!("failed to create session"))?;

    Ok(session_record.id.id.to_string())
}

