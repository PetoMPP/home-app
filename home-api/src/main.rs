use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
    Extension, Router,
};
use database::{user_sessions::UserSessionDatabase, users::UserDatabase, DbManager, DbPool};
use models::{
    auth::{Claims, Token},
    NormalizedString,
};
use r2d2_sqlite::SqliteConnectionManager;
use reqwest::{header::LOCATION, StatusCode};
use services::scanner_service::ScannerService;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{services::ServeDir, trace::TraceLayer};
use website::is_hx_request;

mod database;
mod models;
mod services;
mod website;

refinery::embed_migrations!("migrations");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    // create services
    let mut scanner = ScannerService::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?,
    );
    scanner.init(pool.clone()).await;
    let scanner = Mutex::new(scanner);
    let scanner = Arc::new(scanner);
    // start a task to delete expired tokens
    {
        let pool = pool.clone();
        tokio::spawn(async move {
            loop {
                _ = delete_tokens(&pool).await;
            }
        })
    };
    // build app
    #[allow(unused_mut)]
    let mut app = Router::new()
        // register our webapp
        .route("/", get(website::home::home))
        .route("/sensors", get(website::sensors::sensors))
        .route("/sensors/:host", delete(website::sensors::delete_sensor))
        .route("/sensors/:host/edit", get(website::sensors::edit_sensor))
        .route("/sensors/:host", post(website::sensors::update_sensor))
        .route("/scanner", get(website::scanner::scanner))
        .route("/pair/:host", post(website::scanner::pair_sensor))
        .route("/scan", post(website::scanner::scan))
        .route("/scan/cancel", post(website::scanner::cancel))
        .route("/scan/status", get(website::scanner::status_ws))
        .route("/logout", post(website::login::logout))
        .layer(middleware::from_fn_with_state(pool.clone(), auth))
        .route("/login", get(website::login::login_page))
        .route("/login", post(website::login::login))
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

pub type ApiErrorResponse = (StatusCode, HeaderMap, String);

pub fn into_err_sync(e: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> ApiErrorResponse {
    let box_err = e.into().to_string();
    into_err_str(match box_err.as_str() {
        "" => None,
        e => Some(e),
    })
}

pub fn into_err(e: impl Into<Box<dyn std::error::Error>>) -> ApiErrorResponse {
    let box_err = e.into().to_string();
    into_err_str(match box_err.as_str() {
        "" => None,
        e => Some(e),
    })
}

pub fn into_err_str(e: Option<impl Into<String>>) -> ApiErrorResponse {
    let mut header_map = HeaderMap::new();
    header_map.insert("Hx-Reswap", "innerHTML".parse().unwrap());
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        header_map,
        e.map(|e| e.into()).unwrap_or_default(),
    )
}

async fn auth(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, HeaderMap)> {
    let mut header_map = HeaderMap::new();
    let error = match is_hx_request(&headers) {
        true => {
            header_map.insert("HX-Redirect", "/login".parse().unwrap());

            (StatusCode::UNAUTHORIZED, header_map)
        }
        false => {
            header_map.insert(LOCATION, "/login".parse().unwrap());
            (StatusCode::SEE_OTHER, header_map)
        }
    };
    let (claims, token) = Token::try_from(&headers)
        .and_then(|t| (&t).try_into().map(|c: Claims| (c, t)))
        .map_err(|_| error.clone())?;
    let conn = pool.get().await.map_err(|_| error.clone())?;
    let normalized_name = NormalizedString::new(&claims.sub);
    let _session = conn
        .get_session(normalized_name.clone(), token.clone())
        .await
        .ok()
        .and_then(|s| s)
        .ok_or(error.clone())?;
    if !claims.validate() {
        conn.delete_session(normalized_name, token).await.ok();
        return Err(error);
    }

    Ok(next.run(request).await)
}

async fn delete_tokens(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    let conn = pool.get().await?;
    let sessions = conn.get_sessions().await?;
    let mut invalid_sessions = vec![];
    for session in sessions {
        let Ok(claims): Result<Claims, _> = (&session.token).try_into() else {
            invalid_sessions.push(session);
            continue;
        };
        if !claims.validate() {
            invalid_sessions.push(session);
        }
    }
    if !invalid_sessions.is_empty() {
        return conn.delete_sessions(invalid_sessions).await.map(|_| ());
    }

    Ok(())
}
