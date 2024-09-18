use crate::{
    api_error::ApiErrorResponse,
    models::{RequestData, User},
};
use askama::Template;
use axum::response::Html;

pub mod users;

#[derive(Template)]
#[template(path = "pages/system.html")]
pub struct SystemTemplate {
    pub current_user: Option<User>,
}

#[derive(Template)]
#[template(path = "pages/system-inner.html")]
pub struct SystemInnerTemplate;

pub async fn system(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    if req_data.is_hx_request {
        return Ok(Html(SystemInnerTemplate.render().unwrap()));
    }

    Ok(Html(
        SystemTemplate {
            current_user: req_data.user,
        }
        .render()
        .unwrap(),
    ))
}
