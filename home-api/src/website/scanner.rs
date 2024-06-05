use crate::services::scanner_service::{ScanProgress, ScannerService, ScannerState};
use askama::Template;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    }, http::HeaderMap, response::{Html, IntoResponse}
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

#[derive(Template)]
#[template(path = "pages/scanner.html")]
pub struct ScannerTemplate {
    pub state: ScannerState,
}

#[derive(Template)]
#[template(path = "pages/scanner-inner.html")]
pub struct ScannerInnerTemplate {
    pub state: ScannerState,
}

pub async fn scanner(
    State(scanner): State<Arc<Mutex<ScannerService>>>,
    headers: HeaderMap,
) -> Html<String> {
    let state = scanner.lock().await.state().await;
    match headers.contains_key("Hx-Request") {
        true => Html(ScannerInnerTemplate { state }.render().unwrap()),
        false => Html(ScannerTemplate { state }.render().unwrap()),
    }
}

pub async fn scan(State(scanner): State<Arc<Mutex<ScannerService>>>) -> Html<String> {
    Html(
        ScannerInnerTemplate {
            state: scanner.lock().await.init().await,
        }
        .render()
        .unwrap(),
    )
}

pub async fn cancel(State(scanner): State<Arc<Mutex<ScannerService>>>) -> Html<String> {
    scanner.lock().await.cancel().await;
    Html(
        ScannerInnerTemplate {
            state: scanner.lock().await.state().await,
        }
        .render()
        .unwrap(),
    )
}

#[derive(Template)]
#[template(path = "components/scanner-progress-status.html")]
pub struct ScannerProgressStatusTemplate {
    pub progress: ScanProgress,
}

pub async fn status_ws(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(scanner): State<Arc<Mutex<ScannerService>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_status_socket(socket, addr, scanner.clone()))
}

async fn handle_status_socket(
    mut socket: WebSocket,
    _addr: SocketAddr,
    scanner: Arc<Mutex<ScannerService>>,
) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if let Err(_) = socket.send(Message::Ping(vec![1, 2, 3])).await {
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    let send_task = {
        let scanner = scanner.clone();
        tokio::spawn(async move {
            let mut last_msg = None;
            loop {
                let state = scanner.lock().await.state().await;
                let msg = match &state {
                    ScannerState::Idle(_) | ScannerState::Error(_) => {
                        if socket
                            .send(Message::Text(
                                ScannerInnerTemplate { state }.render().unwrap(),
                            ))
                            .await
                            .is_err()
                        {
                            break;
                        }
                        let _ = socket.send(Message::Close(None)).await;
                        return;
                    }
                    ScannerState::Scanning(progress) => Message::Text(
                        ScannerProgressStatusTemplate {
                            progress: progress.clone(),
                        }
                        .render()
                        .unwrap(),
                    ),
                };
                if let Some(last_msg) = &last_msg {
                    if last_msg == &msg {
                        continue;
                    }
                }
                if let Err(_) = socket.send(msg.clone()).await {
                    break;
                }
                last_msg = Some(msg);
            }
        })
    };

    send_task.await.unwrap();
}
