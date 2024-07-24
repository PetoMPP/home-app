use super::{components::alert::AlertType, is_hx_request};
use crate::{
    database::{sensors::SensorDatabase, DbPool},
    into_err, into_err_str,
    models::{auth::Token, db::SensorEntity, json::Sensor, User},
    services::sensor_service::SensorService,
    ApiErrorResponse,
};
use askama::Template;
use axum::{
    extract::{Path, RawForm},
    http::HeaderMap,
    response::Html,
    Extension,
};
use reqwest::Client;

#[derive(Template)]
#[template(path = "pages/sensors.html")]
pub struct SensorsTemplate {
    pub current_user: Option<User>,
    pub sensors: Vec<SensorEntity>,
    pub action_type: SensorActions,
}

#[derive(Template)]
#[template(path = "pages/sensors-inner.html")]
pub struct SensorsInnerTemplate {
    pub sensors: Vec<SensorEntity>,
    pub action_type: SensorActions,
}

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

#[derive(Template, Default)]
#[template(path = "pages/sensor-edit.html")]
pub struct SensorEditTemplate {
    pub current_user: Option<User>,
    pub sensor: SensorEntity,
    pub alert_message: Option<String>,
    pub alert_type: Option<AlertType>,
}

#[derive(Template, Default)]
#[template(path = "pages/sensor-edit-inner.html")]
pub struct SensorEditInnerTemplate {
    pub sensor: SensorEntity,
    pub alert_message: Option<String>,
    pub alert_type: Option<AlertType>,
}

pub enum SensorActions {
    Overview,
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
            false => SensorActions::Overview,
        }
    }
}

pub async fn sensors(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = pool.get().await.map_err(into_err)?;
    let current_user = Token::get_valid_user(token, &conn)
        .await
        .map_err(into_err)?;
    let sensors = match current_user {
        Some(_) => conn.get_sensors().await.map_err(into_err)?,
        None => vec![],
    };

    match is_hx_request(&headers) {
        true => Ok(Html(
            SensorsInnerTemplate {
                sensors,
                action_type: SensorActions::Overview,
            }
            .render()
            .unwrap(),
        )),
        false => Ok(Html(
            SensorsTemplate {
                sensors,
                current_user,
                action_type: SensorActions::Overview,
            }
            .render()
            .unwrap(),
        )),
    }
}

pub async fn edit_sensor(
    Path(host): Path<String>,
    headers: HeaderMap,
    token: Option<Token>,
    Extension(pool): Extension<DbPool>,
) -> Result<Html<String>, ApiErrorResponse> {
    let host = host.replace('-', ".");
    let conn = pool.get().await.map_err(into_err)?;
    let current_user = Token::get_valid_user(token, &conn)
        .await
        .map_err(into_err)?;
    let sensor = conn
        .get_sensor(&host)
        .await
        .map_err(into_err)?
        .ok_or(into_err_str(Some("Sensor not found")))?;

    Ok(match is_hx_request(&headers) {
        true => Html(
            SensorEditInnerTemplate {
                sensor,
                ..Default::default()
            }
            .render()
            .unwrap(),
        ),
        false => Html(
            SensorEditTemplate {
                current_user,
                sensor,
                ..Default::default()
            }
            .render()
            .unwrap(),
        ),
    })
}

pub async fn update_sensor(
    Path(host): Path<String>,
    token: Option<Token>,
    headers: HeaderMap,
    Extension(pool): Extension<DbPool>,
    form: RawForm,
) -> Result<Html<String>, Html<String>> {
    let sensor = serde_urlencoded::from_bytes::<Sensor>(&form.0)
        .map_err(|e| into_edit_err(e, &headers, None, SensorEntity::default()))?;
    let conn = pool
        .get()
        .await
        .map_err(|e| into_edit_err(e, &headers, None, SensorEntity::default()))?;
    let current_user = Token::get_valid_user(token, &conn)
        .await
        .map_err(|e| into_edit_err(e, &headers, None, SensorEntity::default()))?;
    let host = host.replace('-', ".");
    let sensor_entity = conn
        .get_sensor(&host)
        .await
        .and_then(|s| s.ok_or("Sensor not found".into()))
        .map_err(|e| into_edit_err(e, &headers, current_user.clone(), SensorEntity::default()))?;
    let Some(pair_id) = &sensor_entity.pair_id else {
        return Err(into_edit_err(
            "Sensor not found",
            &headers,
            current_user,
            SensorEntity::default(),
        ));
    };
    let sensor = Client::new()
        .update_sensor(&host, pair_id, sensor)
        .await
        .map_err(|e| into_edit_err(e, &headers, current_user.clone(), sensor_entity.clone()))?;
    let sensor = conn
        .update_sensor(&host, sensor)
        .await
        .map_err(|e| into_edit_err(e, &headers, current_user.clone(), sensor_entity.clone()))?;

    Ok(match is_hx_request(&headers) {
        true => Html(
            SensorEditInnerTemplate {
                sensor,
                alert_message: Some("Sensor updated successfully!".to_string()),
                alert_type: Some(AlertType::Success),
            }
            .render()
            .unwrap(),
        ),
        false => Html(
            SensorEditTemplate {
                current_user,
                sensor,
                alert_message: Some("Sensor updated successfully!".to_string()),
                alert_type: Some(AlertType::Success),
            }
            .render()
            .unwrap(),
        ),
    })
}

fn into_edit_err(
    e: impl Into<Box<dyn std::error::Error>>,
    headers: &HeaderMap,
    current_user: Option<User>,
    sensor: SensorEntity,
) -> Html<String> {
    match is_hx_request(headers) {
        true => Html(
            SensorEditInnerTemplate {
                alert_message: Some(e.into().to_string()),
                alert_type: Some(AlertType::Warning),
                sensor,
            }
            .render()
            .unwrap(),
        ),
        false => Html(
            SensorEditTemplate {
                current_user,
                alert_message: Some(e.into().to_string()),
                alert_type: Some(AlertType::Warning),
                sensor,
            }
            .render()
            .unwrap(),
        ),
    }
}

pub async fn get_sensors(
    Extension(pool): Extension<DbPool>,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = pool.get().await.map_err(into_err)?;
    let sensors = conn.get_sensors().await.map_err(into_err)?;

    Ok(Html(
        SensorRowsTemplate {
            sensors,
            action_type: SensorActions::Overview,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn delete_sensor(
    headers: HeaderMap,
    Path(host): Path<String>,
    Extension(pool): Extension<DbPool>,
) -> Result<Html<String>, ApiErrorResponse> {
    let host = host.replace('-', ".");
    let conn = pool.get().await.map_err(into_err)?;
    conn.delete_sensor(&host).await.map_err(into_err)?;

    Ok(Html(
        SensorRowsTemplate {
            sensors: conn.get_sensors().await.map_err(into_err)?,
            action_type: SensorActions::from(&headers),
        }
        .render()
        .unwrap(),
    ))
}
