use axum::{
    routing::{get, post},
    Extension, Router,
};
use database::{DbManager, DbPool, UserDatabase};
use r2d2_sqlite::SqliteConnectionManager;
use services::scanner_service::ScannerService;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{services::ServeDir, trace::TraceLayer};

mod database;
mod models;
mod services;
mod website;

refinery::embed_migrations!("migrations");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // workaround for running the app from the root of the workspace
    if std::env::current_dir()?.ends_with("home-app") {
        std::env::set_current_dir("home-api")?;
    }
    // initialize tracing
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt().init();
    // run migrations
    {
        let mut conn = r2d2_sqlite::rusqlite::Connection::open("home-api.db")?;
        migrations::runner().run(&mut conn)?;
    }
    // create a connection pool
    let manager = DbManager::new(
        SqliteConnectionManager::file("home-api.db"),
        deadpool::Runtime::Tokio1,
    );
    let pool = DbPool::builder(manager).build()?;
    // ensure we have an admin user
    {
        let conn = pool.get().await?;
        conn.ensure_admin().await?;
    }
    // build app
    let mut scanner = ScannerService::default();
    scanner.init().await;
    let scanner = Mutex::new(scanner);
    let scanner = Arc::new(scanner);
    let mut app = Router::new()
        // register our webapp
        .route("/", axum::routing::get(website::home::home))
        .route("/login", get(website::login::login_page))
        .route("/login", post(website::login::login))
        .route("/sensors", get(website::home::get_sensors))
        .route("/scanner", get(website::scanner::scanner))
        .route("/scan", post(website::scanner::scan))
        .route("/scan/cancel", post(website::scanner::cancel))
        .route("/scan/status", get(website::scanner::status_ws))
        .fallback(website::not_found)
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(Extension(pool))
        .with_state(scanner)
        .layer(TraceLayer::new_for_http());
    #[cfg(debug_assertions)]
    {
        app = app.layer(tower_livereload::LiveReloadLayer::new());
    }

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}
