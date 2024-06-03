use axum::{Extension, Router};
use r2d2_sqlite::SqliteConnectionManager;
use serde_derive::{Deserialize, Serialize};
use sqlite_pool::SqlitePool;
use website::WebappService;

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
    let app = Router::new().register_webapp().layer(Extension(pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Sensor {
    id: String,
    features: u32,
}
