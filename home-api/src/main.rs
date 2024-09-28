use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use database::{users::UserDatabase, DbManager, DbPool};
use models::db::SensorEntity;
use r2d2_sqlite::SqliteConnectionManager;
use services::{scanner_service::ScannerService, sensor_data_service::SensorDataService};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{services::ServeDir, trace::TraceLayer};

mod api_error;
mod auth;
mod database;
mod models;
mod services;
mod ssl;
mod website;

refinery::embed_migrations!("migrations");

const PORT_HTTP: u16 = 3000;
const PORT_HTTPS: u16 = 3001;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt().init();
    // generate a self-signed certificate
    let (cert, pkey) = ssl::generate_ssl()?;
    let cfg = RustlsConfig::from_pem(cert.to_pem()?, pkey.private_key_to_pem_pkcs8()?).await?;
    // spawn a second server to redirect http requests to this server
    ssl::start_https_redirect_server();
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
    // create services
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?;
    let runtime = Arc::new(runtime);
    let mut scanner = ScannerService::<SensorEntity>::new(runtime.clone());
    scanner.init(pool.clone()).await;
    let scanner = Mutex::new(scanner);
    let scanner = Arc::new(scanner);
    let mut data_service = SensorDataService::new(runtime, pool.clone());
    data_service.init().await?;
    let data_service = Mutex::new(data_service);
    let data_service = Arc::new(data_service);
    // start a task to delete expired tokens
    auth::start_user_session_watchdog(pool.clone());
    // build app
    #[allow(unused_mut)]
    let mut app = Router::new()
        // register our webapp
        .route("/", get(website::home::home))
        .route("/sensors", get(website::sensors::sensors))
        .route("/sensors/:host", delete(website::sensors::delete_sensor))
        .route("/sensors/:host", post(website::sensors::update_sensor))
        .route("/sensors/:host/sync", post(website::sensors::sync_sensor))
        .route("/scanner", get(website::scanner::scanner))
        .route("/pair/:host", post(website::scanner::pair_sensor))
        .route("/scan", post(website::scanner::scan))
        .route("/scan/cancel", post(website::scanner::cancel))
        .route("/scan/status", get(website::scanner::status_ws))
        .route("/data", get(website::data::data))
        .route("/data/browse", get(website::data::browse_data::browse_data))
        .route(
            "/data/schedule",
            get(website::data::schedule::data_schedule),
        )
        .route(
            "/data/schedule",
            put(website::data::schedule::create_schedule_entry),
        )
        .route(
            "/data/schedule",
            delete(website::data::schedule::delete_schedule_entry),
        )
        .route("/areas", get(website::areas::areas))
        .route("/areas", put(website::areas::create_area))
        .route("/areas/:id", delete(website::areas::delete_area))
        .route("/areas/:id", post(website::areas::update_area))
        .route("/areas/:id/chart", get(website::areas::area_chart))
        .route("/system", get(website::system::system))
        .route("/system/users", get(website::system::users::users))
        .route("/system/users", put(website::system::users::create_user))
        .route(
            "/system/users",
            post(website::system::users::change_password),
        )
        .route(
            "/system/users/:name",
            delete(website::system::users::delete_user),
        )
        .route("/logout", post(website::login::logout))
        .layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            auth::validate_user_session,
        ))
        .route("/login", get(website::login::login_page))
        .route("/login", post(website::login::login))
        .fallback(website::not_found)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(pool)
        .layer(Extension(data_service))
        .layer(Extension(scanner))
        .layer(TraceLayer::new_for_http());
    #[cfg(debug_assertions)]
    {
        app = app.layer(tower_livereload::LiveReloadLayer::new());
    }

    // run our app with axum_server and rustls
    let addr = SocketAddr::from(([0, 0, 0, 0], PORT_HTTPS));
    tracing::info!("listening on {}", addr);
    Ok(axum_server::bind_rustls(addr, cfg)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?)
}
