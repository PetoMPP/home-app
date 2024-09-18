use crate::{
    api_error::ApiErrorResponse,
    models::{RequestData, User},
};
use askama::Template;
use axum::response::Html;

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

pub async fn data(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    if req_data.is_hx_request {
        return Ok(Html(DataInnerTemplate.render().unwrap()));
    }

    Ok(Html(
        DataTemplate {
            current_user: req_data.user,
        }
        .render()
        .unwrap(),
    ))
}
