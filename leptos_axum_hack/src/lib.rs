use axum::async_trait;
// use crate::{db, AppState};
use axum::body::{Body, Bytes};
use axum::extract::{FromRequest, FromRequestParts, Host, Path, RawQuery, Request, State};
use axum::http::{header, HeaderMap};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
// use happings_lib::db::Session;
use leptos::leptos_server::server_fn_by_path;
use leptos::server_fn::{Encoding, Payload};
use leptos::{create_runtime, provide_context, use_context, ServerFnError};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use scopeguard::defer;
use std::sync::Arc;
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
    // DbError(surrealdb::Error),
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
            // Fail::DbError(e) => e.to_string(),
        };

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(msg))
            .unwrap()
    }
}

pub struct GenericExtractor<T>(T);

// #[async_trait]
// impl<S> FromRequest<S> for ExtraContext<F>
// where
//     S: Send + Sync,
//     Self: Sized,
// {
//     async fn from_request(req: &mut Request, state: &S) {
//         || {
//             if let Ok(session) = get_session(&app_state, /*headers,*/ jar).await {
//                 if let Ok(person) = happenings_lib::person::get_person(session.user.into()).await {
//                     provide_context(person);
//                 }
//             }
//         }
//     }

//     type Rejection = String;
// }

pub async fn handle_server_fns<AS: Clone + Send + 'static, T: Clone + Send + 'static>(
    GenericExtractor::<T>(extra): GenericExtractor<T>,
    Path(fn_name): Path<String>,
    // headers: HeaderMap,
    RawQuery(query): RawQuery,
    State(app_state): State<AS>,
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
        provide_context(ResponseOptions::default());
        provide_context(extra);
        // todo put back
        // if let Ok(session) = get_session(&app_state, /*headers,*/ jar).await {
        //     if let Ok(person) = happenings_lib::person::get_person(session.user.into()).await {
        //         provide_context(person);
        //     }
        // }

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

        let payload = server_fn
            .call((), data.as_ref())
            .await
            .map_err(|e| Fail::ServerFnError(e))?;

        let mut res = Response::builder();
        res = res.status(StatusCode::OK);

        // Add headers from ResponseParts if they exist. These should be added as long
        // as the server function returns an OK response
        let res_options = use_context::<ResponseOptions>();
        let res_options_outer = res_options.unwrap().0;
        let res_options_inner = res_options_outer.read();
        let (status, mut res_headers) =
            (res_options_inner.status, res_options_inner.headers.clone());

        // Override StatusCode if it was set in a Resource or Element
        res = match status {
            Some(status) => res.status(status),
            None => res,
        };

        // This must be after the default referrer
        // redirect so that it overwrites the one above
        if let Some(header_ref) = res.headers_mut() {
            header_ref.extend(res_headers.drain());
        };

        let res =
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
            .expect("could not build response");
        // Ok::<hyper::Response<axum::body::Body>, E>((res)
        Result::<Response<Body>, Fail>::Ok(res)
    };

    let payload_result = get_task_pool().spawn_pinned(task).await;

    let resp = match payload_result {
        Err(e) => Fail::JoinError(e).into_response(),
        Ok(Err(e)) => e.into_response(),
        Ok(Ok(resp)) => resp,
    };

    resp
    // .map_err(|e| );
    // .and_then(std::convert::identity);
}

/// Allows you to override details of the HTTP response like the status code and add Headers/Cookies.
#[derive(Debug, Clone, Default)]
pub struct ResponseOptions(pub Arc<RwLock<ResponseParts>>);

impl ResponseOptions {
    /// A simpler way to overwrite the contents of `ResponseOptions` with a new `ResponseParts`.
    pub fn overwrite(&self, parts: ResponseParts) {
        let mut writable = self.0.write();
        *writable = parts
    }
    /// Set the status of the returned Response.
    pub fn set_status(&self, status: StatusCode) {
        let mut writeable = self.0.write();
        let res_parts = &mut *writeable;
        res_parts.status = Some(status);
    }
    /// Insert a header, overwriting any previous value with the same key.
    pub fn insert_header(&self, key: HeaderName, value: HeaderValue) {
        let mut writeable = self.0.write();
        let res_parts = &mut *writeable;
        res_parts.headers.insert(key, value);
    }
    /// Append a header, leaving any header with the same key intact.
    pub fn append_header(&self, key: HeaderName, value: HeaderValue) {
        let mut writeable = self.0.write();
        let res_parts = &mut *writeable;
        res_parts.headers.append(key, value);
    }
}

/// This struct lets you define headers and override the status of the Response from an Element or a Server Function
/// Typically contained inside of a ResponseOptions. Setting this is useful for cookies and custom responses.
#[derive(Debug, Clone, Default)]
pub struct ResponseParts {
    pub status: Option<StatusCode>,
    pub headers: HeaderMap,
}

impl ResponseParts {
    /// Insert a header, overwriting any previous value with the same key
    pub fn insert_header(&mut self, key: HeaderName, value: HeaderValue) {
        self.headers.insert(key, value);
    }
    /// Append a header, leaving any header with the same key intact
    pub fn append_header(&mut self, key: HeaderName, value: HeaderValue) {
        self.headers.append(key, value);
    }
}

/// Provides an easy way to redirect the user from within a server function. Mimicking the Remix `redirect()`,
/// it sets a StatusCode of 302 and a LOCATION header with the provided value.
/// If looking to redirect from the client, `leptos_router::use_navigate()` should be used instead
pub fn redirect(path: &str) {
    if let Some(response_options) = use_context::<ResponseOptions>() {
        response_options.set_status(StatusCode::FOUND);
        response_options.insert_header(
            header::LOCATION,
            header::HeaderValue::from_str(path).expect("Failed to create HeaderValue"),
        );
    }
}

