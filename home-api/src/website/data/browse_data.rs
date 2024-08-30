use crate::{
    database::{temp_data::TempDataDatabase, DbConn, DbPool},
    into_api_err,
    models::{auth::Token, db::TempDataEntry, User},
    website::is_hx_request,
    ApiErrorResponse,
};
use askama::Template;
use axum::{extract::Query, http::HeaderMap, response::Html, Extension};
use reqwest::StatusCode;
use std::collections::BTreeMap;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Template)]
#[template(path = "pages/data-browse.html")]
pub struct BrowseDataTemplate {
    pub current_user: Option<User>,
    pub page: usize,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Template)]
#[template(path = "pages/data-browse-inner.html")]
pub struct BrowseDataInnerTemplate {
    pub page: usize,
}

pub async fn browse_data(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
    Query(query): Query<BTreeMap<String, String>>,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let is_hx_request = is_hx_request(&headers);
    if let Some(feature) = query.get("feature") {
        let feature = match feature.as_str() {
            "temp" => Some(handle_temp_data(&query, &conn, is_hx_request, &current_user).await),
            _ => None,
        };
        if let Some(feature) = feature {
            return feature;
        }
    }

    if is_hx_request {
        return Ok(Html(BrowseDataInnerTemplate { page: 1 }.render().unwrap()));
    }

    Ok(Html(
        BrowseDataTemplate {
            current_user,
            page: 1,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Template)]
#[template(path = "components/temp-browse.html")]
pub struct TempBrowseTemplate {
    pub items: Vec<TempDataEntry>,
    pub page: usize,
    pub last_page: bool,
}

async fn handle_temp_data(
    query: &BTreeMap<String, String>,
    conn: &DbConn,
    is_hx_request: bool,
    current_user: &Option<User>,
) -> Result<Html<String>, ApiErrorResponse> {
    const PAGE_SIZE: usize = 10;
    let page = query.get("page").and_then(|p| p.parse::<usize>().ok());
    let offset = page.map(|p| (p - 1) * PAGE_SIZE);
    let items = into_api_err(
        conn.get_temp_data(Option::<&'static str>::None, Some(PAGE_SIZE + 1), offset)
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let last_page = items.get(PAGE_SIZE).is_none();
    if is_hx_request {
        return Ok(Html(
            TempBrowseTemplate {
                items: items.into_iter().take(PAGE_SIZE).collect(),
                page: page.unwrap_or(1),
                last_page,
            }
            .render()
            .unwrap(),
        ));
    }

    Ok(Html(
        BrowseDataTemplate {
            current_user: current_user.clone(),
            page: page.unwrap_or(1),
        }
        .render()
        .unwrap(),
    ))
}
