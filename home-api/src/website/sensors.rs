use askama::Template;
use axum::{extract::Path, http::HeaderMap, response::Html, Extension};
use reqwest::StatusCode;

use crate::{
    database::{sensors::SensorDatabase, DbPool},
    models::db::SensorEntity,
};

#[derive(Template)]
#[template(path = "components/sensor-rows.html")]
pub struct SensorRowsTemplate {
    pub sensors: Vec<SensorEntity>,
    pub action_type: SensorActions,
}

#[derive(Template)]
#[template(path = "components/sensor-row.html")]
pub struct SensorRowTemplate {
    pub sensor: SensorEntity,
    pub action_type: SensorActions,
}

pub enum SensorActions {
    Home,
    Scanner,
}

impl From<&HeaderMap> for SensorActions {
    fn from(headers: &HeaderMap) -> Self {
        match headers
            .get("Hx-Current-Url")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.ends_with("scanner"))
            .unwrap_or_default()
        {
            true => SensorActions::Scanner,
            false => SensorActions::Home,
        }
    }
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

    Ok(Html(
        SensorRowsTemplate {
            sensors,
            action_type: SensorActions::Home,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn delete_sensor(
    headers: HeaderMap,
    Path(host): Path<String>,
    Extension(pool): Extension<DbPool>,
) -> Result<Html<String>, (StatusCode, String)> {
    let host = host.replace("-", ".");
    let conn = pool
        .get()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    conn.delete_sensor(&host)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(
        SensorRowsTemplate {
            sensors: conn
                .get_sensors()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            action_type: SensorActions::from(&headers),
        }
        .render()
        .unwrap(),
    ))
}
