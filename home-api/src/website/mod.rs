use askama::Template;
use axum::{http::StatusCode, response::Html};

pub mod home;
pub mod scanner;

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
