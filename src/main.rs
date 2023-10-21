use std::net::SocketAddr;

use axum::{Router, response::Html, routing::get};
use tokio::{signal, sync::broadcast};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "spreading_fire=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (tx, _rx) = broadcast::channel(100);
    let tx_shutdown = tx.clone();

    let app = Router::new()
        .route("/", get(index));

    let address = SocketAddr::from(([0,0,0,0],3000));
    tracing::debug!("listing on {}", address);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
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

async fn index() -> Html<&'static str> {
    Html(std::include_str!("../public/index.html"))
}

