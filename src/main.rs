mod config;
mod db;
mod error_handling;
mod oauth;
mod server;

use anyhow::anyhow;
use axum::{extract::{Path, Request, State}, http::header::{self}, http::{HeaderMap, StatusCode}, middleware::{self, Next}, response::{IntoResponse, Response}, routing::get, Json, Router};
use dotenv::dotenv;
use error_handling::AppError;
use figment::{providers::{Env, Format, Serialized, Toml}, Figment};
use rust_embed::RustEmbed;
use surrealdb::{engine::any::{connect, Any}, opt::auth::Root, Surreal};
use tracing::*;

async fn log_errors(req: Request, next: Next) -> Result<Response, StatusCode> {
    let res = next.run(req).await;

    if res.status().is_success() {
        return Ok(res);
    }

    let (parts, body) = res.into_parts();

    let body_bytes = axum::body::to_bytes(body, 4 * 1024)
        .await
        .inspect_err(|_| warn!("error response with large body"))
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    let body_str = std::str::from_utf8(&body_bytes)
        .inspect_err(|_| warn!("error response with non-utf body"))
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    warn!("error response: {}", body_str);

    Ok(Response::from_parts(parts, axum::body::Body::from(body_bytes)))
}

#[derive(Clone)]
struct AppState {
    config: config::Config,
    db: Surreal<Any>,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config: config::Config = Figment::from(Serialized::defaults(config::Config::default()))
        .merge(Toml::file("happenings.toml"))
        .merge(Env::prefixed("APP_"))
        .extract()?;

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(Level::WARN.into()) // Default level for all modules
        .parse_lossy("happenings=debug,tower_http=trace");

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(filter)
        .init();

    info!("Something is happening..");
    info!("Connecting to database at {:?}", &config.db.endpoint);
    info!("{:?}", std::env::current_dir());
    let db = connect(&config.db.endpoint).await?;

    if let Some(config::Credentials::Root {
        ref username,
        ref password,
    }) = config.db.credentials
    {
        db.signin(Root { username, password }).await?;
    };

    db.use_ns(&config.db.namespace)
        .use_db(&config.db.database)
        .await?;

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/app.wasm", get(wasm_handler))
        .route("/app.js", get(js_handler))
        .route("/static/*path", get(static_handler))
        .route("/api/login", get(oauth::login_handler))
        .route("/api/user", get(user_handler))
        .route("/api/oauth_return", get(oauth::oauth_return))
        .fallback(get(root_handler))
        .layer(middleware::from_fn(log_errors))
        .with_state(AppState { db, config });

    server::serve(app).await;
    info!("graceful shutdown complete");
    Ok(())
}

async fn user_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let session_id = headers
        .get("Authorization")
        .ok_or(anyhow!("No Authorization Header"))?
        .to_str()?;

    let session: db::Session = app_state
        .db
        .select(("session", session_id))
        .await?
        .ok_or(anyhow!("no session with id"))?;

    if chrono::Utc::now() > session.expires_at {
        Err(anyhow!("session expired"))?
    }

    let user: db::Person = app_state
        .db
        .select(session.user)
        .await?
        .ok_or(anyhow!("no user matching id in session"))?;

    let resp = common::UserInfoReponse {
        id: user.id.id.to_string(),
        given_name: user.given_name,
        family_name: user.family_name,
        picture: user.picture,
        email: user.email,
    };

    Ok(Json(resp))
}

async fn root_handler() -> impl IntoResponse {
    let path = "index.html";
    match Static::get(path) {
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
    }
}

async fn wasm_handler() -> impl IntoResponse {
    let bytes = include_bytes!(env!("HAPPENINGS_WASM"));
    ([(header::CONTENT_TYPE, "application/wasm")], bytes)
}

async fn js_handler() -> impl IntoResponse {
    let bytes = include_bytes!(env!("HAPPENINGS_JS"));
    ([(header::CONTENT_TYPE, "text/javascript")], bytes)
}

#[derive(RustEmbed)]
#[folder = "static"]
struct Static;

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match Static::get(&path) {
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
    }
}

