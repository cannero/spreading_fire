use std::{net::SocketAddr, sync::Arc};

use axum::{Router, response::IntoResponse, routing::get, http::StatusCode, Json};
use serde::Serialize;
use tokio::{signal, sync::broadcast};
use tower_http::{trace::TraceLayer, services::{ServeDir, ServeFile}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod appstate;
mod websocket;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "spreading_fire=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (tx, _rx) = broadcast::channel(100);
    let tx_shutdown = tx.clone();
    let app_state = Arc::new(appstate::AppState::new(tx));

    let app = Router::new()
        .nest_service("/", ServeDir::new("public").not_found_service(ServeFile::new("public/index.html")))
        .route("/frames", get(frames))
        .route("/_websocket", get(websocket::websocket_handler))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    let address = SocketAddr::from(([0,0,0,0],3000));
    tracing::debug!("listing on {}", address);

    axum::Server::bind(&address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal(tx_shutdown))
        .await
        .unwrap();
}

async fn shutdown_signal(tx: broadcast::Sender<String>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl_c");
    };

    #[cfg(unix)]
    let terminate = async {
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

    let msg = "server shutdown".to_string();
    tracing::info!(msg);
    let _ = tx.send(msg);
}

#[derive(Debug, Serialize, Clone)]
struct Frame {
    id: u32,
    text: String,
}

async fn frames() ->  impl IntoResponse {

    let frame = Frame{
        id: 12,
        text: "hello".to_string(),
    };

    (StatusCode::OK , Json(frame))
}



