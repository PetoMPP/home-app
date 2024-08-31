use super::sensors::{SensorActions, SensorTemplate};
use crate::{
    database::{sensors::SensorDatabase, DbPool},
    into_api_err,
    models::{
        db::{SensorEntity, SensorFeatures},
        RequestData, User,
    },
    services::{
        scanner_service::{ScannerService, ScannerState},
        sensor_service::SensorService,
    },
    ApiErrorResponse,
};
use askama::Template;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Path, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    Extension,
};
use reqwest::{Client, StatusCode};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Template)]
#[template(path = "pages/scanner.html")]
pub struct ScannerTemplate {
    pub current_user: Option<User>,
    pub state: ScannerState<SensorEntity>,
    pub action_type: SensorActions,
    pub sensors: Vec<SensorEntity>,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Template)]
#[template(path = "pages/scanner-ws.html")]
pub struct ScannerWsTemplate {
    pub state: ScannerState<SensorEntity>,
    pub action_type: SensorActions,
    pub sensors: Vec<SensorEntity>,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Template)]
#[template(path = "pages/scanner-content.html")]
pub struct ScannerContentTemplate {
    pub state: ScannerState<SensorEntity>,
    pub action_type: SensorActions,
    pub sensors: Vec<SensorEntity>,
}

pub async fn scanner(
    req_data: RequestData,
    Extension(scanner): Extension<Arc<Mutex<ScannerService<SensorEntity>>>>,
) -> Result<Html<String>, ApiErrorResponse> {
    let state = scanner.lock().await.state().await;
    let sensors = state.scanned();
    if req_data.is_hx_request {
        return Ok(Html(
            ScannerWsTemplate {
                state,
                action_type: SensorActions::Scanner,
                sensors,
            }
            .render()
            .unwrap(),
        ));
    }

    Ok(Html(
        ScannerTemplate {
            state,
            current_user: req_data.user,
            action_type: SensorActions::Scanner,
            sensors,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn scan(
    Extension(scanner): Extension<Arc<Mutex<ScannerService<SensorEntity>>>>,
    Extension(pool): Extension<DbPool>,
) -> Html<String> {
    let state = scanner.lock().await.init(pool).await;
    Html(
        ScannerContentTemplate {
            sensors: state.scanned(),
            state,
            action_type: SensorActions::Scanner,
        }
        .render()
        .unwrap(),
    )
}

pub async fn cancel(
    Extension(scanner): Extension<Arc<Mutex<ScannerService<SensorEntity>>>>,
) -> Html<String> {
    scanner.lock().await.cancel().await;
    let state = scanner.lock().await.state().await;
    Html(
        ScannerContentTemplate {
            sensors: state.scanned(),
            state,
            action_type: SensorActions::Scanner,
        }
        .render()
        .unwrap(),
    )
}

pub async fn pair_sensor(
    req_data: RequestData,
    Path(host): Path<String>,
) -> Result<Html<String>, ApiErrorResponse> {
    let host = host.replace('-', ".");
    let sensor = into_api_err(
        Client::new().pair(&host).await,
        StatusCode::UNAUTHORIZED,
        &req_data,
    )?;
    let sensor = into_api_err(
        req_data.conn.create_sensor(sensor).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    Ok(Html(
        SensorTemplate {
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
    Extension(scanner): Extension<Arc<Mutex<ScannerService<SensorEntity>>>>,
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
                let state = scanner.lock().await.state().await;
                let msg = Message::Text(
                    ScannerContentTemplate {
                        sensors: state.scanned(),
                        state,
                        action_type: SensorActions::Scanner,
                    }
                    .render()
                    .unwrap(),
                );
                if last_msg.as_ref() != Some(&msg) {
                    last_msg = Some(msg.clone());
                    if socket.send(msg.clone()).await.is_err() {
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(650)).await;
            }
        })
    };

    send_task.await.unwrap();
}
