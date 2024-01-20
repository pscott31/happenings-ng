use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

use tracing::*;

pub async fn log_errors(req: Request, next: Next) -> Result<Response, StatusCode> {
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
