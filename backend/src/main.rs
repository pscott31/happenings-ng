mod middleware;
mod server;

use axum::{body::Body, extract::{Host, Path, Request, State}, http::{header, StatusCode}, response::IntoResponse, routing::{get, post}, Router};
use axum_extra::extract::CookieJar;
use common::config::Config;
use common::{axum::LoggedInUser, AppState};
use dotenv::dotenv;
use figment::{providers::{Env, Format, Serialized, Toml}, Figment};
use leptos::provide_context;
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

    if let Some(common::config::Credentials::Root {
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

pub async fn my_handler(
    jar: CookieJar,
    host: Host,
    logged_in_user: Option<LoggedInUser>,
    State(state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    let additional_context = move || {
        provide_context(logged_in_user.clone());
        provide_context(jar.clone());
        provide_context(host.clone());
        provide_context(state.clone());
    };

    leptos_axum::handle_server_fns_with_context(additional_context, req).await
}
fn build_app(db: Surreal<Any>, config: Config) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/app.wasm", get(wasm_handler))
        .route("/app.js", get(js_handler))
        .route("/static/*path", get(static_handler))
        .route("/api/*fn_name", post(my_handler))
        .route("/api/*fn_name", get(my_handler))
        .fallback(get(root_handler))
        .with_state(common::AppState { db, config })
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
#[folder = "../static"]
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

