use crate::{
    database::DbPool,
    into_api_err,
    models::{auth::Token, User},
    ApiErrorResponse,
};
use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::Html,
    Extension,
};

pub mod components;
pub mod data;
pub mod home;
pub mod login;
pub mod scanner;
pub mod sensors;

pub fn is_hx_request(headers: &HeaderMap) -> bool {
    headers.contains_key("Hx-Request")
}

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
    headers: HeaderMap,
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
) -> Result<(StatusCode, Html<String>), ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    Ok((
        StatusCode::NOT_FOUND,
        match is_hx_request(&headers) {
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
        },
    ))
}
