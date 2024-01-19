use crate::db::Session;
use crate::{db, AppState};
use axum::body::{Body, Bytes};
use axum::extract::{Host, Path, RawQuery, Request, State};
// use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use axum_macros::debug_handler;
use leptos::leptos_server::server_fn_by_path;
use leptos::server_fn::{Encoding, Payload};
use leptos::{create_runtime, provide_context, ServerFnError};
use once_cell::sync::OnceCell;
use scopeguard::defer;
use std::thread::available_parallelism;
use tokio_util::task::LocalPoolHandle;

const MAX_BODY_SIZE: usize = 1 * 1024 * 1024;

fn get_task_pool() -> LocalPoolHandle {
    static LOCAL_POOL: OnceCell<LocalPoolHandle> = OnceCell::new();
    LOCAL_POOL
        .get_or_init(|| {
            tokio_util::task::LocalPoolHandle::new(
                available_parallelism().map(Into::into).unwrap_or(1),
            )
        })
        .clone()
}

pub enum Fail {
    BadServerPath(String),
    ServerFnError(ServerFnError),
    JoinError(tokio::task::JoinError),
    // BadAuthHeader,
    // NoAuthHeader,
    NoSession,
    NoAuthCookie,
    SessionExpired,
    DbError(surrealdb::Error),
}

impl IntoResponse for Fail {
    fn into_response(self) -> Response {
        let msg = match self {
            Fail::BadServerPath(p) => format!("no server function '{p}' found"),
            Fail::ServerFnError(e) => e.to_string(),
            Fail::JoinError(e) => e.to_string(),
            // Fail::NoAuthHeader => "no authorization header found".to_string(),
            // Fail::BadAuthHeader => "authorization header malformed".to_string(),
            Fail::NoAuthCookie => "no authorization cookie found".to_string(),
            Fail::NoSession => "no session found".to_string(),
            Fail::SessionExpired => "session expired".to_string(),
            Fail::DbError(e) => e.to_string(),
        };

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(msg))
            .unwrap()
    }
}
pub async fn get_session(
    app_state: &AppState,
    // headers: HeaderMap,
    jar: CookieJar,
) -> Result<Session, Fail> {
    // TODO: Figure out if we're going to use headers or cookies
    // let session_id = match headers.get("Authorization") {
    //     Some(header) => header.to_str().map_err(|e| Fail::BadAuthHeader)?,
    //     None => return Err(Fail::NoAuthHeader),
    // };

    let Some(session_cookie) = jar.get("session_id") else {
        return Err(Fail::NoAuthCookie);
    };

    let session_id = session_cookie.value();

    let session: Option<db::Session> = app_state
        .db
        .select(("session", session_id))
        .await
        .map_err(|e| Fail::DbError(e))?;

    let session = match session {
        Some(session) => session,
        None => return Err(Fail::NoSession),
    };

    if chrono::Utc::now() > session.expires_at {
        return Err(Fail::SessionExpired);
    }
    Ok(session)
}

#[debug_handler]
pub async fn handle_server_fns(
    Path(fn_name): Path<String>,
    // headers: HeaderMap,
    RawQuery(query): RawQuery,
    State(app_state): State<AppState>,
    host: Host,
    jar: CookieJar,
    req: Request<Body>,
) -> impl IntoResponse {
    let fn_name = fn_name
        .strip_prefix('/')
        .map(|fn_name| fn_name.to_string())
        .unwrap_or(fn_name);

    // The future returned server_fn_by_path is !Send, so we can't just await it
    let task = || async move {
        let runtime = create_runtime();
        defer! {
            runtime.dispose();
        }
        provide_context(app_state.clone());
        provide_context(host);
        if let Ok(session) = get_session(&app_state, /*headers,*/ jar).await {
            if let Ok(person) = happenings::person::get_person(session.user.into()).await {
                provide_context(person);
            }
        }

        let server_fn = server_fn_by_path(fn_name.as_str()).ok_or(Fail::BadServerPath(fn_name))?;

        let (_parts, body) = req.into_parts();
        let body = axum::body::to_bytes(body, MAX_BODY_SIZE)
            .await
            .unwrap_or_default();

        let query: Bytes = query.unwrap_or("".to_string()).into();
        let data = match &server_fn.encoding() {
            Encoding::Url | Encoding::Cbor => body,
            Encoding::GetJSON | Encoding::GetCBOR => query,
        };

        server_fn
            .call((), data.as_ref())
            .await
            .map_err(|e| Fail::ServerFnError(e))
    };

    let payload = get_task_pool()
        .spawn_pinned(task)
        .await
        .map_err(|e| Fail::JoinError(e))
        .and_then(std::convert::identity);

    let payload = match payload {
        Ok(pl) => pl,
        Err(e) => return e.into_response(),
    };

    let res = Response::builder();
    match payload {
        Payload::Binary(data) => res
            .header("Content-Type", "application/cbor")
            .body(Body::from(data)),
        Payload::Url(data) => res
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(data)),
        Payload::Json(data) => res
            .header("Content-Type", "application/json")
            .body(Body::from(data)),
    }
    .expect("could not build response")
}

