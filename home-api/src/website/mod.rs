use crate::{
    models::{RequestData, User},
    ApiErrorResponse,
};
use askama::Template;
use axum::{http::StatusCode, response::Html};

pub mod areas;
pub mod components;
pub mod data;
pub mod home;
pub mod login;
pub mod scanner;
pub mod sensors;

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

pub async fn not_found(
    req_data: RequestData,
) -> Result<(StatusCode, Html<String>), ApiErrorResponse> {
    Ok((
        StatusCode::NOT_FOUND,
        match req_data.is_hx_request {
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
                    current_user: req_data.user,
                    status: StatusCode::NOT_FOUND,
                    message: "Not Found".to_string(),
                }
                .render()
                .unwrap(),
            ),
        },
    ))
}
