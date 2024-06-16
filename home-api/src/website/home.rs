use super::{is_hx_request, sensors::SensorActions};
use crate::{
    database::{sensors::SensorDatabase, DbPool},
    models::{auth::Token, db::SensorEntity, User},
};
use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::Html,
    Extension,
};

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub current_user: Option<User>,
    pub sensors: Vec<SensorEntity>,
    pub action_type: SensorActions,
}

#[derive(Template)]
#[template(path = "pages/home-inner.html")]
pub struct HomeInnerTemplate {
    pub sensors: Vec<SensorEntity>,
    pub action_type: SensorActions,
}

pub async fn home(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, (StatusCode, String)> {
    let conn = pool
        .get()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let current_user = Token::get_valid_user(token, &conn).await?;
    let sensors = match current_user {
        Some(_) => conn
            .get_sensors()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        None => vec![],
    };

    match is_hx_request(&headers) {
        true => Ok(Html(
            HomeInnerTemplate {
                sensors,
                action_type: SensorActions::Home,
            }
            .render()
            .unwrap(),
        )),
        false => Ok(Html(
            HomeTemplate {
                sensors,
                current_user,
                action_type: SensorActions::Home,
            }
            .render()
            .unwrap(),
        )),
    }
}
