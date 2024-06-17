use super::{is_hx_request, sensors::SensorActions};
use crate::{
    database::{sensors::SensorDatabase, DbPool},
    into_err,
    models::{auth::Token, User},
    services::{
        scanner_service::{ScanProgress, ScannerService, ScannerState},
        sensor_service::SensorService,
    },
    website::sensors::SensorRowTemplate,
    ApiErrorResponse,
};
use askama::Template;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Path, State, WebSocketUpgrade,
    },
    http::HeaderMap,
    response::{Html, IntoResponse},
    Extension,
};
use reqwest::Client;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

#[derive(Template)]
#[template(path = "pages/scanner.html")]
pub struct ScannerTemplate {
    pub current_user: Option<User>,
    pub state: ScannerState,
    pub action_type: SensorActions,
}

#[derive(Template)]
#[template(path = "pages/scanner-inner.html")]
pub struct ScannerInnerTemplate {
    pub state: ScannerState,
    pub action_type: SensorActions,
}

pub async fn scanner(
    State(scanner): State<Arc<Mutex<ScannerService>>>,
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = pool.get().await.map_err(into_err)?;
    let current_user = Token::get_valid_user(token, &conn)
        .await
        .map_err(into_err)?;
    let mut state = scanner.lock().await.state().await;
    if let ScannerState::Idle(Some(result)) = &mut state {
        result.check_sensors(&pool).await.map_err(into_err)?;
    }
    Ok(match is_hx_request(&headers) {
        true => Html(
            ScannerInnerTemplate {
                state,
                action_type: SensorActions::Scanner,
            }
            .render()
            .unwrap(),
        ),
        false => Html(
            ScannerTemplate {
                state,
                current_user,
                action_type: SensorActions::Scanner,
            }
            .render()
            .unwrap(),
        ),
    })
}

pub async fn scan(State(scanner): State<Arc<Mutex<ScannerService>>>) -> Html<String> {
    Html(
        ScannerInnerTemplate {
            state: scanner.lock().await.init().await,
            action_type: SensorActions::Scanner,
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
            action_type: SensorActions::Scanner,
        }
        .render()
        .unwrap(),
    )
}

pub async fn pair_sensor(
    Extension(pool): Extension<DbPool>,
    Path(host): Path<String>,
) -> Result<Html<String>, ApiErrorResponse> {
    let host = host.replace('-', ".");
    let sensor = Client::new().pair(&host).await.map_err(into_err)?;
    let conn = pool.get().await.map_err(into_err)?;
    let sensor = conn.create_sensor(sensor).await.map_err(into_err)?;

    Ok(Html(
        SensorRowTemplate {
            sensor,
            action_type: SensorActions::Scanner,
        }
        .render()
        .unwrap(),
    ))
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
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_err() {
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
                                ScannerInnerTemplate {
                                    state,
                                    action_type: SensorActions::Scanner,
                                }
                                .render()
                                .unwrap(),
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
                if socket.send(msg.clone()).await.is_err() {
                    break;
                }
                last_msg = Some(msg);
            }
        })
    };

    send_task.await.unwrap();
}
