use axum::{http::StatusCode, routing::get, Extension, Json, Router};
use r2d2_sqlite::SqliteConnectionManager;
use serde_derive::{Deserialize, Serialize};

type SqlitePool = r2d2::Pool<SqliteConnectionManager>;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    // create a connection pool
    let man = SqliteConnectionManager::file("home-api.db");
    let pool: SqlitePool = r2d2::Pool::new(man).expect("Failed to create pool");
    // build app
    let app = Router::new()
        .route("/", get(root))
        .layer(Extension(pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize)]
struct Sensor {
    id: String,
    features: u32,
}

// basic handler that responds with a static string
async fn root(Extension(pool): Extension<SqlitePool>) -> Result<Json<Vec<Sensor>>, StatusCode> {
    let conn = pool.get().unwrap();
    let mut stmt = conn.prepare("SELECT * FROM sensors").unwrap();
    let sensor_iter = stmt
        .query_map([], |row| {
            Ok(Sensor {
                id: row.get(0)?,
                features: row.get(1)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|s| s.ok())
        .collect::<Vec<_>>();

    Ok(Json(sensor_iter))
}
