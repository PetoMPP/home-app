use askama::Template;
use axum::{http::StatusCode, response::Html, routing::get};
use home::{get_sensors, home};
use tower_http::services::ServeFile;

mod home;

pub trait WebappService {
    fn register_webapp(self) -> Self;
}

impl<S> WebappService for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn register_webapp(self) -> Self {
        self.route("/", axum::routing::get(home))
            .route("/sensors", get(get_sensors))
            .fallback(not_found)
            .nest_service("/output.css", ServeFile::new("output.css"))
            .nest_service("/htmx.min.js", ServeFile::new("htmx.min.js"))
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
