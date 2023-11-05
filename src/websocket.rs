use std::{net::SocketAddr, time::{Duration, Instant}, sync::Arc};

use axum::{extract::{WebSocketUpgrade, ConnectInfo, ws::{WebSocket, Message}, State}, response::IntoResponse};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::{time::sleep, sync::mpsc};

use crate::appstate::AppState;

pub async fn websocket_handler(ws: WebSocketUpgrade, ConnectInfo(addr): ConnectInfo<SocketAddr>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, addr, state))
}

async fn long_running_calc(tx: mpsc::Sender<String>,){
    sleep(Duration::from_secs(5)).await;
    let now = Instant::now();
    let _ = tx.send(format!("calculation done at {now:?}")).await;
    tracing::debug!("long_running_calc end");
}

async fn websocket(stream: WebSocket, who: SocketAddr, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let (websocket_tx, mut websocket_rx) = mpsc::channel(5);

    let mut rx = state.create_rx();

    tracing::info!("websocket join {who}");

    let websocket_from_global_tx = websocket_tx.clone();
    let mut send_task_from_global = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if websocket_from_global_tx.send(msg).await.is_err(){
                break;
            }
        }
    });

    let stream_tx = state.clone_tx();

    let mut recv_task_client = tokio::spawn(async move {
        let mut stream_recv_task = tokio::spawn(async move {
            while let Some(Ok(Message::Text(message))) = receiver.next().await {
                if message.starts_with("[Run calculation]"){
                    state.abort_if_exists(&who).await;
                    let calculation_tx = websocket_tx.clone();
                    let calc_task = tokio::spawn(async {
                        long_running_calc(calculation_tx).await;
                    });
                    state.add(who, calc_task).await;
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
