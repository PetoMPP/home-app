use super::components::alert::{AlertTemplate, AlertType};
use crate::{
    api_err,
    database::sensors::SensorDatabase,
    into_api_err,
    models::{
        db::{SensorEntity, SensorFeatures},
        json::SensorFormData,
        RequestData, User,
    },
    services::sensor_service::SensorService,
    ApiErrorResponse,
};
use askama::Template;
use axum::{
    extract::{Path, RawForm},
    http::HeaderMap,
    response::Html,
};
use reqwest::{Client, StatusCode};

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
#[template(path = "components/sensor.html")]
pub struct SensorTemplate {
    pub sensor: SensorEntity,
    pub action_type: SensorActions,
}

pub fn sensor_style(sensor: &SensorEntity) -> &'static str {
    if sensor.features.is_empty() {
        return "bg-neutral border-neutral-content text-neutral-content opacity-70";
    }

    "bg-base-300 border-base-content text-base-content"
}

#[derive(Template, Default)]
#[template(path = "pages/sensor-edit.html")]
pub struct SensorEditTemplate {
    pub current_user: Option<User>,
    pub sensor: SensorEntity,
}

#[derive(Template, Default)]
#[template(path = "pages/sensor-edit-inner.html")]
pub struct SensorEditInnerTemplate {
    pub sensor: SensorEntity,
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

pub async fn sensors(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    let sensors = into_api_err(
        req_data.conn.get_sensors().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    match req_data.is_hx_request {
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
                current_user: req_data.user,
                action_type: SensorActions::Overview,
            }
            .render()
            .unwrap(),
        )),
    }
}

pub async fn edit_sensor(
    req_data: RequestData,
    Path(host): Path<String>,
) -> Result<Html<String>, ApiErrorResponse> {
    let host = host.replace('-', ".");
    let sensor = into_api_err(
        req_data.conn.get_sensor(&host).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let Some(sensor) = sensor else {
        return api_err("Sensor not found", StatusCode::NOT_FOUND, &req_data);
    };
    Ok(match req_data.is_hx_request {
        true => Html(SensorEditInnerTemplate { sensor }.render().unwrap()),
        false => Html(
            SensorEditTemplate {
                current_user: req_data.user,
                sensor,
            }
            .render()
            .unwrap(),
        ),
    })
}

pub async fn update_sensor(
    req_data: RequestData,
    Path(host): Path<String>,
    form: RawForm,
) -> Result<Html<String>, ApiErrorResponse> {
    let sensor = match serde_urlencoded::from_bytes::<SensorFormData>(&form.0) {
        Ok(sensor) => sensor,
        Err(e) => {
            return api_err(
                format!("Form decoding error: {}", e),
                StatusCode::BAD_REQUEST,
                &req_data,
            );
        }
    };
    let host = host.replace('-', ".");
    let sensor_entity = into_api_err(
        req_data.conn.get_sensor(&host).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let Some(sensor_entity) = sensor_entity else {
        return api_err("Sensor not found", StatusCode::NOT_FOUND, &req_data);
    };
    let Some(pair_id) = &sensor_entity.pair_id else {
        return api_err("Sensor not found", StatusCode::NOT_FOUND, &req_data);
    };
    let sensor = into_api_err(
        Client::new().update_sensor(&host, pair_id, sensor).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    into_api_err(
        req_data.conn.update_sensor(&host, sensor).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    Ok(Html(
        AlertTemplate {
            alert_message: Some("Sensor updated successfully!".to_string()),
            alert_type: Some(AlertType::Success),
        }
        .render()
        .unwrap(),
    ))
}

pub async fn delete_sensor(
    req_data: RequestData,
    Path(host): Path<String>,
) -> Result<Html<String>, ApiErrorResponse> {
    let host = host.replace('-', ".");
    let affected = into_api_err(
        req_data.conn.delete_sensor(&host).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    if affected == 0 {
        return api_err("Sensor not found", StatusCode::NOT_FOUND, &req_data);
    }

    Ok(Html(
        SensorsInnerTemplate {
            sensors: into_api_err(
                req_data.conn.get_sensors().await,
                StatusCode::INTERNAL_SERVER_ERROR,
                &req_data,
            )?,
            action_type: SensorActions::Overview,
        }
        .render()
        .unwrap(),
    ))
}
