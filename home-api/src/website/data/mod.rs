use super::is_hx_request;
use crate::{
    database::DbPool,
    into_api_err,
    models::{auth::Token, User},
    ApiErrorResponse,
};
use askama::Template;
use axum::{http::HeaderMap, response::Html, Extension};
use reqwest::StatusCode;

pub mod browse_data;
pub mod schedule;

#[derive(Template)]
#[template(path = "pages/data.html")]
pub struct DataTemplate {
    pub current_user: Option<User>,
}

#[derive(Template)]
#[template(path = "pages/data-inner.html")]
pub struct DataInnerTemplate;

pub async fn data(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;

    if is_hx_request(&headers) {
        return Ok(Html(DataInnerTemplate.render().unwrap()));
    }

    Ok(Html(
        DataTemplate {
            current_user: current_user.clone(),
        }
        .render()
        .unwrap(),
    ))
}
