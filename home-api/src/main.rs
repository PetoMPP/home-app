use axum::{
    extract::State,
    routing::{get, post},
    Extension, Router,
};
use r2d2_sqlite::SqliteConnectionManager;
use services::scanner_service::ScannerService;
use sqlite_pool::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use website::WebappService;

mod models;
mod services;
mod sqlite_pool;
mod website;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    // create a connection pool
    let man = SqliteConnectionManager::file("home-api.db");
    let pool = SqlitePool(r2d2::Pool::new(man).expect("Failed to create pool"));
    // build app
    let mut scanner = ScannerService::default();
    scanner.init().await;
    let scanner = Mutex::new(scanner);
    let scanner = Arc::new(scanner);
    let mut app = Router::new()
        .register_webapp()
        .route("/api/v1/scanner/collect", get(scanner_collect))
        .route("/api/v1/scanner/init", post(scanner_init))
        .layer(Extension(pool))
        .with_state(scanner);
    #[cfg(debug_assertions)]
    {
        app = app.layer(tower_livereload::LiveReloadLayer::new());
    }

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn scanner_collect(State(scanner): State<Arc<Mutex<ScannerService>>>) -> String {
    format!("{:?}", scanner.lock().await.state().await)
}

async fn scanner_init(State(scanner): State<Arc<Mutex<ScannerService>>>) -> String {
    format!("{:?}", scanner.lock().await.init().await)
}
