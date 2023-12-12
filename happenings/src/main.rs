mod server;

use axum::{
    http::header::{self},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use dotenv::dotenv;
use tracing::*;

// pub mod fileserv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(Level::WARN.into()) // Default level for all modules
        .parse_lossy("happenings=debug");

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(filter)
        .init();

    info!("Off we go!");

    let app = Router::new()
        .route("/", get(root_handler))
        // .route("/app.wasm", get(wasm_handler))
        // .route("/app.js", get(js_handler))
        ;

    server::serve(app).await;
    info!("graceful shutdown complete");
}

async fn root_handler() -> impl IntoResponse {
    Html(
        r#"<!DOCTYPE html>
<html>
    <head>
        <link rel="preload" href="/app.wasm" as="fetch" type="application/wasm" crossorigin="">
        <link rel="modulepreload" href="/app.js"></head>
    </head>
    <body>
        <script type="module">import init from '/app.js';init('/app.wasm');</script>
        <p>Hello, World!</p>
    </body>
</html>
    "#,
    )
}

// async fn wasm_handler() -> impl IntoResponse {
//     // let bytes = include_bytes!("../../target/wasm32-unknown-unknown/debug/frontend.wasm");
//     let bytes = include_bytes!(env!("HAPPENINGS_WASM"));
//     ([(header::CONTENT_TYPE, "application/wasm")], bytes)
// }

// async fn js_handler() -> impl IntoResponse {
//     // let bytes = include_bytes!("../../target/wasm32-unknown-unknown/debug/frontend.wasm");
//     let bytes = include_bytes!(env!("HAPPENINGS_JS"));
//     ([(header::CONTENT_TYPE, "	text/javascript")], bytes)
// }
