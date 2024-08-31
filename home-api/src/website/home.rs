use crate::{
    models::{RequestData, User},
    ApiErrorResponse,
};
use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub current_user: Option<User>,
}

#[derive(Template)]
#[template(path = "pages/home-inner.html")]
pub struct HomeInnerTemplate;

pub async fn home(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    if req_data.is_hx_request {
        return Ok(Html(HomeInnerTemplate.render().unwrap()));
    }

    Ok(Html(
        HomeTemplate {
            current_user: req_data.user,
        }
        .render()
        .unwrap(),
    ))
}
