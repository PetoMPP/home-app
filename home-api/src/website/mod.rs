use askama::Template;
use axum::{http::{HeaderMap, StatusCode}, response::Html};

pub mod home;
pub mod scanner;
pub mod login;

#[derive(Template)]
#[template(path = "pages/error.html")]
pub struct ErrorTemplate {
    pub status: StatusCode,
    pub message: String,
}

#[derive(Template)]
#[template(path = "pages/error-inner.html")]
pub struct ErrorInnerTemplate {
    pub status: StatusCode,
    pub message: String,
}

pub async fn not_found(headers: HeaderMap) -> Html<String> {
    match headers.contains_key("Hx-Request") {
        true => Html(
            ErrorInnerTemplate {
                status: StatusCode::NOT_FOUND,
                message: "Not Found".to_string(),
            }
            .render()
            .unwrap(),
        ),
        false => Html(
            ErrorTemplate {
                status: StatusCode::NOT_FOUND,
                message: "Not Found".to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}
