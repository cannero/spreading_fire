use std::{net::SocketAddr, sync::Arc, time::Instant};

use axum::{Router, response::IntoResponse, routing::get, http::StatusCode, Json, extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}, ConnectInfo}};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Serialize;
use tokio::{signal, sync::{broadcast, mpsc}, time::{sleep, Duration}};
use tower_http::{trace::TraceLayer, services::{ServeDir, ServeFile}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


struct AppState {
    tx: broadcast::Sender<String>,
}

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
    let app_state = Arc::new(AppState{tx});

    let app = Router::new()
        .nest_service("/", ServeDir::new("public").not_found_service(ServeFile::new("public/index.html")))
        .route("/frames", get(frames))
        .route("/_websocket", get(websocket_handler))
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

async fn websocket_handler(ws: WebSocketUpgrade, ConnectInfo(addr): ConnectInfo<SocketAddr>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, addr, state))
}

async fn long_running_calc(tx: mpsc::Sender<String>,){
    sleep(Duration::from_secs(3)).await;
    let now = Instant::now();
    let _ = tx.send(format!("calculation done at {now:?}")).await;
    tracing::debug!("long_running_calc end");
}

async fn websocket(stream: WebSocket, who: SocketAddr, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let (websocket_tx, mut websocket_rx) = mpsc::channel(5);

    let mut rx = state.tx.subscribe();

    tracing::info!("websocket join {who}");

    let websocket_from_global_tx = websocket_tx.clone();
    let mut send_task_from_global = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if websocket_from_global_tx.send(msg).await.is_err(){
                break;
            }
        }
    });

    let stream_tx = state.tx.clone();

    let mut recv_task_client = tokio::spawn(async move {
        let mut stream_recv_task = tokio::spawn(async move {
            while let Some(Ok(Message::Text(message))) = receiver.next().await {
                if message.starts_with("[Run calculation]"){
                    let calculation_tx = websocket_tx.clone();
                    tokio::spawn(async {
                        long_running_calc(calculation_tx).await;
                    });
                    tracing::info!("calc started");
                } else {
                    let _ = stream_tx.send(message.to_string());
                }
            }
        });

        let mut websocket_recv_task = tokio::spawn(async move {
            while let Some(res) = websocket_rx.recv().await {
                tracing::debug!("websocket feedback {res}");
                if sender.send(Message::Text(res)).await.is_err() {
                    break;
                }
            }
        });

        tokio::select! {
            _ = (&mut stream_recv_task) => websocket_recv_task.abort(),
            _ = (&mut websocket_recv_task) => stream_recv_task.abort(),
        }
    });

    tokio::select! {
        _ = (&mut send_task_from_global) => recv_task_client.abort(),
        _ = (&mut recv_task_client) => send_task_from_global.abort(),
    }

    tracing::info!("websocket disconnected");
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



