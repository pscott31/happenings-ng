use axum::body::Body;

use crate::person::DbPerson;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

use crate::{auth::session::{DbSession, Session, SessionId}, AppState};
use crate::{person::Person};

use axum::{async_trait, extract::{FromRequestParts}, http::request::Parts, http::StatusCode, response::{IntoResponse, Response}};
use axum_extra::extract::CookieJar;

// use server_fns::get_session;

pub enum Fail {
    BadServerPath(String),
    // ServerFnError(ServerFnError),
    JoinError(tokio::task::JoinError),
    // BadAuthHeader,
    // NoAuthHeader,
    NoUser,
    NoSession,
    NoAuthCookie,
    SessionExpired,
    DbError(surrealdb::Error),
}

impl IntoResponse for Fail {
    fn into_response(self) -> Response {
        let msg = match self {
            Fail::BadServerPath(p) => format!("no server function '{p}' found"),
            Fail::JoinError(e) => e.to_string(),
            Fail::NoAuthCookie => "no authorization cookie found".to_string(),
            Fail::NoSession => "no session found".to_string(),
            Fail::SessionExpired => "session expired".to_string(),
            Fail::DbError(e) => e.to_string(),
            Fail::NoUser => "session user not found".to_string(),
        };

        // TODO - better codes for different errors
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(msg))
            .unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct LoggedInUser(pub Person);
#[async_trait]
impl FromRequestParts<AppState> for LoggedInUser
where
    Self: Sized,
{
    type Rejection = Fail;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let SessionWrapper(session) = SessionWrapper::from_request_parts(parts, state).await?;

        let person: DbPerson = state
        .db
        .select(&session.user)
        .await
        .map_err(Fail::DbError)?
        .ok_or(Fail::NoUser)?;

        Ok(LoggedInUser(person.into()))
    }
}

pub struct SessionWrapper(Session);
#[async_trait]
impl FromRequestParts<AppState> for SessionWrapper
where
    Self: Sized,
{
    type Rejection = Fail;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        let session_cookie = jar.get("session_id").ok_or(Fail::NoAuthCookie)?;
        let session_id: SessionId = session_cookie.value().into();

        let session: DbSession = state
            .db
            .select(session_id)
            .await
            .map_err(Fail::DbError)?
            .ok_or(Fail::NoSession)?;

        if chrono::Utc::now() > session.expires_at {
            return Err(Fail::SessionExpired);
        }

        Ok(SessionWrapper(session.into()))
    }
}

}}

