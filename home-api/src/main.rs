use axum::{
    routing::{get, post},
    Extension, Router,
};
use r2d2_sqlite::SqliteConnectionManager;
use services::scanner_service::ScannerService;
use sqlite_pool::SqlitePool;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{services::ServeFile, trace::TraceLayer};

mod models;
mod services;
mod sqlite_pool;
mod website;

#[tokio::main]
async fn main() {
    // initialize tracing
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt().init();
    // create a connection pool
    let man = SqliteConnectionManager::file("home-api.db");
    let pool = SqlitePool(r2d2::Pool::new(man).expect("Failed to create pool"));
    // build app
    let mut scanner = ScannerService::default();
    scanner.init().await;
    let scanner = Mutex::new(scanner);
    let scanner = Arc::new(scanner);
    let mut app = Router::new()
        // register our webapp
        .route("/", axum::routing::get(website::home::home))
        .route("/sensors", get(website::home::get_sensors))
        .route("/scanner", get(website::scanner::scanner))
        .route("/scan", post(website::scanner::scan))
        .route("/scan/cancel", post(website::scanner::cancel))
        .route("/scan/status", get(website::scanner::status_ws))
        .fallback(website::not_found)
        .nest_service("/output.css", ServeFile::new("output.css"))
        .nest_service("/htmx.min.js", ServeFile::new("htmx.min.js"))
        .nest_service("/loading-states.js", ServeFile::new("loading-states.js"))
        .nest_service("/ws.js", ServeFile::new("ws.js"))
        .layer(Extension(pool))
        .with_state(scanner)
        .layer(TraceLayer::new_for_http());
    #[cfg(debug_assertions)]
    {
        app = app.layer(tower_livereload::LiveReloadLayer::new());
    }

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
