mod auth;
mod error_handling;
mod middleware;
mod server;
mod server_fns;

use crate::error_handling::AppError;
use anyhow::anyhow;
use axum::{extract::{Path, State}, http::header::{self}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{get, post}, Json, Router};
use dotenv::dotenv;
use figment::{providers::{Env, Format, Serialized, Toml}, Figment};
use happenings::config::Config;
use happenings::db;
use happenings::AppState;
use rust_embed::RustEmbed;
use surrealdb::{engine::any::{connect, Any}, opt::auth::Root, Surreal};
use tracing::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    info!("initalising happenings");
    dotenv().ok();
    let config = load_config()?;
    setup_logging();
    let db = connect_db(&config).await?;
    let app = build_app(db, config).layer(axum::middleware::from_fn(middleware::log_errors));

    server::serve(app).await;
    info!("graceful shutdown complete");
    Ok(())
}

fn load_config() -> anyhow::Result<Config> {
    let cfg = Figment::from(Serialized::defaults(Config::default()))
        .merge(Toml::file("happenings.toml"))
        .merge(Env::prefixed("APP_"))
        .extract()?;
    Ok(cfg)
}

fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(Level::WARN.into()) // Default level for all modules
        .parse_lossy("happenings=debug,tower_http=trace");

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(filter)
        .init();
}

async fn connect_db(config: &Config) -> anyhow::Result<Surreal<Any>> {
    info!("connecting to database at {:?}", &config.db.endpoint);
    let db = connect(&config.db.endpoint).await?;

    if let Some(happenings::config::Credentials::Root {
        ref username,
        ref password,
    }) = config.db.credentials
    {
        db.signin(Root { username, password }).await?;
    };

    db.use_ns(&config.db.namespace)
        .use_db(&config.db.database)
        .await?;

    // let schema = include_str!("schema.surql");
    // db.query(schema).await?.check()?;

    Ok(db)
}

fn build_app(db: Surreal<Any>, config: Config) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/app.wasm", get(wasm_handler))
        .route("/app.js", get(js_handler))
        .route("/static/*path", get(static_handler))
        .route("/api/auth/oauth/link", post(auth::oauth::login_handler))
        .route("/api/auth/password/signin", post(auth::password::signin))
        .route("/api/auth/password/signup", post(auth::password::signup))
        .route("/api/user_exists", post(user_exists_handler))
        .route("/api/user", post(user_handler)) // TODO: rename current_user?
        .route("/api/auth/oauth/return", post(auth::oauth::oauth_return))
        .route("/api/*fn_name", post(server_fns::handle_server_fns))
        .fallback(get(root_handler))
        .with_state(happenings::AppState { db, config })
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
        .ok_or(anyhow!("no session with id {session_id}"))?;

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
        picture: user.picture.unwrap_or_default(),
        email: user.email,
    };

    Ok(Json(resp))
}

async fn user_exists_handler(
    State(app_state): State<AppState>,
    query: Json<common::Email>,
) -> Result<impl IntoResponse, AppError> {
    let people: Vec<db::Person> = app_state
        .db
        .query("SELECT * FROM person where email=$email;")
        .bind(("email", &query.0.email))
        .await?
        .take(0)?;
    Ok(Json(!people.is_empty()))
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
    // In debug mode, read the file at runtime
    #[cfg(debug_assertions)]
    let content = std::fs::read(env!("HAPPENINGS_WASM")).expect("Failed to read file");

    // In release mode, embed the file at compile time
    #[cfg(not(debug_assertions))]
    let content = include_bytes!(env!("HAPPENINGS_WASM"));

    ([(header::CONTENT_TYPE, "application/wasm")], content)
}

async fn js_handler() -> impl IntoResponse {
    // In debug mode, read the file at runtime
    #[cfg(debug_assertions)]
    let content = std::fs::read(env!("HAPPENINGS_JS")).expect("Failed to read file");

    // In release mode, embed the file at compile time
    #[cfg(not(debug_assertions))]
    let content = include_bytes!(env!("HAPPENINGS_JS"));

    ([(header::CONTENT_TYPE, "text/javascript")], content)
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

#[cfg(test)]
mod tests {
    use super::*;
    use ::axum_test::TestServer;
    use happenings::config::Config;

    #[tokio::test]
    async fn it_works() -> anyhow::Result<()> {
        let cfg = test_config();

        let db = connect_db(&cfg).await?;
        let app = build_app(db, cfg);
        let server = TestServer::new(app).unwrap();

        let user = common::NewUser {
            given_name: "fred".to_string(),
            family_name: "bloggs".to_string(),
            email: "fred@bloggs.com".to_string(),
            password: "super_secret".to_string(),
            phone: Some("123".to_string()),
        };

        let creds = common::EmailPassword {
            email: user.email.clone(),
            password: user.password.clone(),
        };

        let email = common::Email {
            email: user.email.clone(),
        };

        // User should not exit to begin with
        let resp = server.post("/api/user_exists").json(&email).await;
        assert_eq!(resp.status_code(), StatusCode::OK);
        assert!(!resp.json::<bool>());

        // Try logging in before we've made a user
        let resp = server.post("/api/auth/password/signin").json(&creds).await;
        // TODO: Should be UNAUTHORIZED
        assert_eq!(resp.status_code(), StatusCode::INTERNAL_SERVER_ERROR);

        // Create a user
        let resp = server.post("/api/auth/password/signup").json(&user).await;
        assert_eq!(resp.status_code(), StatusCode::OK);

        // User should exist now
        let resp = server.post("/api/user_exists").json(&email).await;
        assert_eq!(resp.status_code(), StatusCode::OK);
        assert!(resp.json::<bool>());

        // Should be able to log in now.
        let resp = server.post("/api/auth/password/signin").json(&creds).await;
        assert_eq!(resp.status_code(), StatusCode::OK);

        // assert_true()

        Ok(())
    }

    fn test_config() -> Config {
        Config {
            db: happenings::config::DB {
                endpoint: "mem://".to_string(),
                credentials: None,
                namespace: "test".to_string(),
                database: "test".to_string(),
            },
            ..Config::default()
        }
    }
}

