use crate::{
    database::{sensors::SensorDatabase, DbPool},
    models::{auth::Token, User},
};
use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::Html,
    Extension,
};
use home_common::models::Sensor;

use super::should_load_inner;

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub current_user: Option<User>,
    pub sensors: Vec<Sensor>,
}

#[derive(Template)]
#[template(path = "pages/home-inner.html")]
pub struct HomeInnerTemplate {
    pub sensors: Vec<Sensor>,
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

    match should_load_inner(&headers) {
        true => Ok(Html(HomeInnerTemplate { sensors }.render().unwrap())),
        false => Ok(Html(
            HomeTemplate {
                sensors,
                current_user,
            }
            .render()
            .unwrap(),
        )),
    }
}

#[derive(Template)]
#[template(path = "components/sensor-rows.html")]
pub struct SensorRowsTemplate {
    pub sensors: Vec<Sensor>,
}

pub async fn get_sensors(
    Extension(pool): Extension<DbPool>,
) -> Result<Html<String>, (StatusCode, String)> {
    let conn = pool
        .get()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let sensors = conn
        .get_sensors()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(SensorRowsTemplate { sensors }.render().unwrap()))
}
