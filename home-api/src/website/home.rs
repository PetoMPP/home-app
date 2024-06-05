use crate::{models::Sensor, sqlite_pool::SqlitePool};
use askama::Template;
use axum::{http::StatusCode, response::Html, Extension};

#[derive(Template, Default)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub sensors: Vec<Sensor>,
}

pub async fn home(Extension(pool): Extension<SqlitePool>) -> Result<Html<String>, StatusCode> {
    let sensors = pool
        .get_sensors()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(HomeTemplate { sensors }.render().unwrap()))
}

#[derive(Template)]
#[template(path = "components/sensor-rows.html")]
pub struct SensorRowsTemplate {
    pub sensors: Vec<Sensor>,
}

pub async fn get_sensors(
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, StatusCode> {
    let sensors = pool
        .get_sensors()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(SensorRowsTemplate { sensors }.render().unwrap()))
}
