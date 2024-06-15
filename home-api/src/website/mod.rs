use crate::models::User;
use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::Html,
};

pub mod home;
pub mod login;
pub mod menu;
pub mod scanner;

#[derive(Template)]
#[template(path = "pages/error.html")]
pub struct ErrorTemplate {
    pub current_user: Option<User>,
    pub status: StatusCode,
    pub message: String,
}

#[derive(Template)]
#[template(path = "pages/error-inner.html")]
pub struct ErrorInnerTemplate {
    pub status: StatusCode,
    pub message: String,
}

pub async fn not_found(headers: HeaderMap, current_user: Option<User>) -> Html<String> {
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
                current_user,
                status: StatusCode::NOT_FOUND,
                message: "Not Found".to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}
