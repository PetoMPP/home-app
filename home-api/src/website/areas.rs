use super::is_hx_request;
use crate::{
    database::DbPool,
    into_api_err,
    models::{auth::Token, db::AreaEntity, User},
    ApiErrorResponse,
};
use askama::Template;
use axum::{http::HeaderMap, response::Html, Extension};
use reqwest::StatusCode;

#[derive(Template)]
#[template(path = "pages/areas.html")]
pub struct AreasTemplate {
    pub current_user: Option<User>,
    pub areas: Vec<AreaEntity>,
}

#[derive(Template)]
#[template(path = "pages/areas-inner.html")]
pub struct AreasInnerTemplate {
    pub areas: Vec<AreaEntity>,
}

pub async fn areas(
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
        return Ok(Html(AreasInnerTemplate { areas: vec![] }.render().unwrap()));
    }

    Ok(Html(
        AreasTemplate {
            current_user: current_user.clone(),
            areas: vec![],
        }
        .render()
        .unwrap(),
    ))
}
