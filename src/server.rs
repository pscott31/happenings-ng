use axum::extract::Request;
use hyper::{body::Incoming, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, signal};
use tower::Service;
use tracing::*;

pub async fn serve(app: axum::Router) {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    loop {
        let (socket, remote_addr) = tokio::select! {
            result = listener.accept() => {
                result.unwrap()
            }
            _ = shutdown_signal() => {
                debug!("signal received, not accepting new connections");
                break;
            }
        };
        let socket = TokioIo::new(socket);
        let tower_service = app.clone();

        tokio::spawn(async move {
            let hyper_service =
                service_fn(move |request: Request<Incoming>| tower_service.clone().call(request));

            let conn = http1::Builder::new().serve_connection(socket, hyper_service);
            let mut conn = std::pin::pin!(conn);

            loop {
                tokio::select! {
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            debug!("failed to serve connection: {err:#}");
                        }
                        break;
                    }
                    _ = shutdown_signal()  => {
                        debug!("signal received, starting graceful shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }

            debug!("connection {remote_addr} closed");
        });
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate =
        async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

