use crate::database::{DbPool, SensorDatabase};
use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::Html,
    Extension,
};
use home_common::models::Sensor;

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub sensors: Vec<Sensor>,
}

#[derive(Template)]
#[template(path = "pages/home-inner.html")]
pub struct HomeInnerTemplate {
    pub sensors: Vec<Sensor>,
}

pub async fn home(
    Extension(pool): Extension<DbPool>,
    headers: HeaderMap,
) -> Result<Html<String>, (StatusCode, String)> {
    let conn = pool
        .get()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let sensors = conn
        .get_sensors()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match headers.contains_key("Hx-Request") {
        true => Ok(Html(HomeInnerTemplate { sensors }.render().unwrap())),
        false => Ok(Html(HomeTemplate { sensors }.render().unwrap())),
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
