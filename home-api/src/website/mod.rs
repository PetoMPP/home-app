use axum::{http::StatusCode, response::Html, routing::get};
use home::{get_sensors, home};
use tower_http::services::ServeFile;
use askama::Template;

mod home;

pub trait WebappService {
    fn register_webapp(self) -> Self;
}

impl WebappService for axum::Router {
    fn register_webapp(self) -> Self {
        self.route("/", axum::routing::get(home))
            .route("/sensors", get(get_sensors))
            .fallback(not_found)
            .nest_service("/output.css", ServeFile::new("output.css"))
    }
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub status: StatusCode,
    pub message: String,
}

pub async fn not_found() -> Html<String> {
    Html(
        ErrorTemplate {
            status: StatusCode::NOT_FOUND,
            message: "Not Found".to_string(),
        }
        .render()
        .unwrap(),
    )
}
