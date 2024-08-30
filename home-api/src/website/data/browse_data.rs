use crate::{
    database::{temp_data::TempDataDatabase, DbPool},
    into_api_err,
    models::{auth::Token, db::TempDataEntry, User},
    website::is_hx_request,
    ApiErrorResponse,
};
use askama::Template;
use axum::{extract::Query, http::HeaderMap, response::Html, Extension};
use reqwest::StatusCode;
use serde::Deserialize;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Template)]
#[template(path = "pages/data-browse.html")]
pub struct BrowseDataTemplate {
    pub current_user: Option<User>,
    pub feature: Option<String>,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Template)]
#[template(path = "pages/data-browse-inner.html")]
pub struct BrowseDataInnerTemplate {
    pub feature: Option<String>,
}

#[derive(Deserialize)]
pub struct BrowseDataQuery {
    feature: Option<String>,
    page: Option<usize>,
}

pub async fn browse_data(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
    Query(query): Query<BrowseDataQuery>,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let is_hx_request = is_hx_request(&headers);
    if let Some(feature) = query.feature.as_ref() {
        let feature = match feature.as_str() {
            "temp" => Some(handle_temp_data(query.page, &pool, is_hx_request, &current_user).await),
            _ => None,
        };
        if let Some(feature) = feature {
            return feature;
        }
    }

    if is_hx_request {
        return Ok(Html(
            BrowseDataInnerTemplate {
                feature: query.feature,
            }
            .render()
            .unwrap(),
        ));
    }

    Ok(Html(
        BrowseDataTemplate {
            current_user,
            feature: query.feature,
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
    page: Option<usize>,
    pool: &DbPool,
    is_hx_request: bool,
    current_user: &Option<User>,
) -> Result<Html<String>, ApiErrorResponse> {
    const PAGE_SIZE: usize = 10;
    let offset = page.map(|p| (p - 1) * PAGE_SIZE);
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
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
            feature: Some(
                TempBrowseTemplate {
                    items: items.into_iter().take(PAGE_SIZE).collect(),
                    page: page.unwrap_or(1),
                    last_page,
                }
                .render()
                .unwrap(),
            ),
        }
        .render()
        .unwrap(),
    ))
}
