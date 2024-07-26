use super::{is_hx_request, sensors::SensorActions};
use crate::{
    database::{sensors::SensorDatabase, DbPool},
    into_err, into_err_sync,
    models::{auth::Token, db::SensorEntity, User},
    services::{
        scanner_service::{ScannerService, ScannerState},
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
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[derive(Template)]
#[template(path = "pages/scanner.html")]
pub struct ScannerTemplate {
    pub current_user: Option<User>,
    pub state: ScannerState<SensorEntity>,
    pub action_type: SensorActions,
    pub sensors: Vec<SensorEntity>,
}

#[derive(Template)]
#[template(path = "pages/scanner-inner.html")]
pub struct ScannerInnerTemplate {
    pub state: ScannerState<SensorEntity>,
    pub action_type: SensorActions,
    pub sensors: Vec<SensorEntity>,
}

pub async fn scanner(
    State(scanner): State<Arc<Mutex<ScannerService<SensorEntity>>>>,
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = pool.get().await.map_err(into_err)?;
    let current_user = Token::get_valid_user(token, &conn)
        .await
        .map_err(into_err)?;
    let state = scanner.lock().await.state().await;
    let sensors = state.scanned();
    Ok(match is_hx_request(&headers) {
        true => Html(
            ScannerInnerTemplate {
                state,
                action_type: SensorActions::Scanner,
                sensors,
            }
            .render()
            .unwrap(),
        ),
        false => Html(
            ScannerTemplate {
                state,
                current_user,
                action_type: SensorActions::Scanner,
                sensors,
            }
            .render()
            .unwrap(),
        ),
    })
}

pub async fn scan(
    State(scanner): State<Arc<Mutex<ScannerService<SensorEntity>>>>,
    Extension(pool): Extension<DbPool>,
) -> Html<String> {
    let state = scanner.lock().await.init(pool).await;
    Html(
        ScannerInnerTemplate {
            sensors: state.scanned(),
            state,
            action_type: SensorActions::Scanner,
        }
        .render()
        .unwrap(),
    )
}

pub async fn cancel(
    State(scanner): State<Arc<Mutex<ScannerService<SensorEntity>>>>,
) -> Html<String> {
    scanner.lock().await.cancel().await;
    let state = scanner.lock().await.state().await;
    Html(
        ScannerInnerTemplate {
            sensors: state.scanned(),
            state,
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
    let sensor = Client::new().pair(&host).await.map_err(into_err_sync)?;
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

pub async fn status_ws(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(scanner): State<Arc<Mutex<ScannerService<SensorEntity>>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_status_socket(socket, addr, scanner.clone()))
}

async fn handle_status_socket(
    mut socket: WebSocket,
    _addr: SocketAddr,
    scanner: Arc<Mutex<ScannerService<SensorEntity>>>,
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
                tokio::time::sleep(Duration::from_millis(650)).await;
                let state = scanner.lock().await.state().await;
                let msg = Message::Text(
                    ScannerInnerTemplate {
                        sensors: state.scanned(),
                        state,
                        action_type: SensorActions::Scanner,
                    }
                    .render()
                    .unwrap(),
                );
                if last_msg.as_ref() == Some(&msg) {
                    continue;
                }
                last_msg = Some(msg.clone());
                if socket.send(msg.clone()).await.is_err() {
                    break;
                }
            }
        })
    };

    send_task.await.unwrap();
}
