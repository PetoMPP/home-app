use crate::{
    database::DbPool, into_err, models::{auth::Token, User}, ApiErrorResponse
};
use askama::Template;
use axum::{http::HeaderMap, response::Html, Extension};

use super::is_hx_request;

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub current_user: Option<User>,
}

#[derive(Template)]
#[template(path = "pages/home-inner.html")]
pub struct HomeInnerTemplate;

pub async fn home(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = pool.get().await.map_err(into_err)?;
    let current_user = Token::get_valid_user(token, &conn)
        .await
        .map_err(into_err)?;

    if is_hx_request(&headers) {
        return Ok(Html(HomeInnerTemplate.render().unwrap()));
    }
    Ok(Html(
        HomeTemplate {
            current_user: current_user.clone(),
        }
        .render()
        .unwrap(),
    ))
}
