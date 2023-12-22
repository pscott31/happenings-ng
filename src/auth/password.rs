use crate::error_handling::AppError;
use anyhow::anyhow;
use axum::{extract::State, response::IntoResponse, Json};
use rand::distributions::{Alphanumeric, DistString};
use sha256::Sha256Digest;

use tracing::*;

use crate::{db, AppState};

use super::create_session;

pub async fn signup(
    State(app_state): State<AppState>,
    Json(payload): Json<common::NewUser>,
) -> Result<impl IntoResponse, AppError> {
    info!("new user: {:?}", payload);
    let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let hash = make_hash(&salt, payload.password);

    let _foo: db::Record = app_state
        .db
        .create("person")
        .content(db::NewPerson {
            given_name: payload.given_name,
            family_name: payload.family_name,
            picture: None,
            email: payload.email,
            phone: payload.phone,
            credentials: db::Credentials::Password { hash, salt },
        })
        .await?
        .pop()
        .ok_or(anyhow!("failed to create new user"))?;
    Ok(Json(()))
}

pub async fn signin(
    State(app_state): State<AppState>,
    Json(payload): Json<common::EmailPassword>,
) -> Result<impl IntoResponse, AppError> {
    let people: Vec<db::Person> = app_state
        .db
        .query("select * from person where email=$email")
        .bind(("email", &payload.email))
        .await?
        .take(0)?;

    let person = people
        .first()
        .ok_or(anyhow!("no user with email {}", payload.email))?;

    match &person.credentials {
        db::Credentials::OAuth => Err(anyhow!(
            "user with email {} exists, but associated with oauth login",
            payload.email
        ))?,
        db::Credentials::Password { hash, salt } => {
            make_hash(salt, payload.password)
                .eq(hash)
                .then_some(())
                .ok_or(anyhow!("incorrect password for '{}'", payload.email))?;
        }
    }

    let session_id = create_session(app_state.db, person.id.id.to_string()).await?;
    Ok(Json(common::Session { id: session_id }))
}

fn make_hash<T, S>(salt: T, pw: S) -> String
where
    T: AsRef<str>,
    S: AsRef<str>,
{
    let salted = format!("{}{}", salt.as_ref(), pw.as_ref());
    Sha256Digest::digest(salted)
}

