use crate::{
    database::DbPool,
    models::{auth::Token, User},
};
use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::Html,
    Extension,
};

pub mod home;
pub mod login;
pub mod scanner;

pub fn should_load_inner(headers: &HeaderMap) -> bool {
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
) -> Result<Html<String>, (StatusCode, String)> {
    let conn = pool
        .get()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let current_user = Token::get_valid_user(token, &conn).await?;
    Ok(match should_load_inner(&headers) {
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
    })
}
