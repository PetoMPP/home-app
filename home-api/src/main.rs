use askama::Template;
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Router,
};
use database::{user_sessions::UserSessionDatabase, users::UserDatabase, DbManager, DbPool};
use models::{
    auth::{Claims, Token},
    db::SensorEntity,
    NormalizedString, RequestData,
};
use r2d2_sqlite::SqliteConnectionManager;
use reqwest::{header::LOCATION, StatusCode};
use services::{scanner_service::ScannerService, sensor_data_service::SensorDataService};
use std::{fmt::Display, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{services::ServeDir, trace::TraceLayer};
use website::{components::alert::AlertTemplate, ErrorTemplate};

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
        .route("/logout", post(website::login::logout))
        .layer(middleware::from_fn_with_state(pool.clone(), auth))
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

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}

pub type ApiErrorResponse = (StatusCode, HeaderMap, axum::response::Html<String>);

pub fn api_err<T>(
    error: impl Into<String>,
    code: StatusCode,
    req_data: &RequestData,
) -> Result<T, ApiErrorResponse> {
    if req_data.is_hx_request {
        return Err((
            code,
            headers(),
            axum::response::Html(
                AlertTemplate {
                    alert_message: Some(error.into()),
                    alert_type: Some(code.into()),
                }
                .render()
                .unwrap(),
            ),
        ));
    }

    Err((
        code,
        headers(),
        axum::response::Html(
            ErrorTemplate {
                current_user: req_data.user.clone(),
                status: code,
                message: error.into(),
            }
            .render()
            .unwrap(),
        ),
    ))
}

pub fn into_api_err<T>(
    result: Result<T, impl Display>,
    code: StatusCode,
    req_data: &RequestData,
) -> Result<T, ApiErrorResponse> {
    if req_data.is_hx_request {
        return result.map_err(|e| {
            (
                code,
                headers(),
                axum::response::Html(
                    AlertTemplate {
                        alert_message: Some(e.to_string()),
                        alert_type: Some(code.into()),
                    }
                    .render()
                    .unwrap(),
                ),
            )
        });
    }

    result.map_err(|e| {
        (
            code,
            headers(),
            axum::response::Html(
                ErrorTemplate {
                    current_user: req_data.user.clone(),
                    status: code,
                    message: e.to_string(),
                }
                .render()
                .unwrap(),
            ),
        )
    })
}

fn headers() -> HeaderMap {
    let mut header_map = HeaderMap::new();
    header_map.insert("Hx-Retarget", "#alert-element".parse().unwrap());
    header_map.insert("Hx-Reswap", "outerHTML".parse().unwrap());
    header_map
}

async fn auth(
    req_data: RequestData,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, HeaderMap)> {
    let mut header_map = HeaderMap::new();
    let error = match req_data.is_hx_request {
        true => {
            header_map.insert("HX-Redirect", "/login".parse().unwrap());

            (StatusCode::UNAUTHORIZED, header_map)
        }
        false => {
            header_map.insert(LOCATION, "/login".parse().unwrap());
            (StatusCode::SEE_OTHER, header_map)
        }
    };
    let (claims, token) = Token::try_from(&req_data.headers)
        .and_then(|t| (&t).try_into().map(|c: Claims| (c, t)))
        .map_err(|_| error.clone())?;
    let normalized_name = NormalizedString::new(&claims.sub);
    let _session = req_data
        .conn
        .get_session(normalized_name.clone(), token.clone())
        .await
        .ok()
        .and_then(|s| s)
        .ok_or(error.clone())?;
    if !claims.validate() {
        req_data
            .conn
            .delete_session(normalized_name, token)
            .await
            .ok();
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
