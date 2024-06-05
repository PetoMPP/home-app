use crate::services::scanner_service::{ScannerService, ScannerState};
use askama::Template;
use axum::{extract::State, response::Html};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Template)]
#[template(path = "scanner.html")]
pub struct ScannerTemplate {
    pub state: ScannerState,
}

pub async fn scanner(State(scanner): State<Arc<Mutex<ScannerService>>>) -> Html<String> {
    Html(
        ScannerTemplate {
            state: scanner.lock().await.state().await,
        }
        .render()
        .unwrap(),
    )
}

#[derive(Template)]
#[template(path = "components/scanner-inner.html")]
pub struct ScannerInnerTemplate {
    pub state: ScannerState,
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