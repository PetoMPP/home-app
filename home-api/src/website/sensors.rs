use super::{
    components::alert::{AlertTemplate, AlertType},
    is_hx_request,
};
use crate::{
    api_err,
    database::{sensors::SensorDatabase, DbPool},
    into_api_err,
    models::{
        auth::Token,
        db::{SensorEntity, SensorFeatures},
        json::SensorFormData,
        User,
    },
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

pub fn sensor_name(sensor: &SensorEntity) -> String {
    if sensor.features.is_empty() {
        return format!("❔ {}", sensor.name);
    }
    let mut features = sensor.features;
    let mut next_features = sensor.features;
    let mut str = String::new();
    next_features.remove(SensorFeatures::TEMPERATURE);
    if !features.difference(next_features).is_empty() {
        features = next_features;
        str.push_str("🌡️");
    }
    next_features.remove(SensorFeatures::MOTION);
    if !features.difference(next_features).is_empty() {
        features = next_features;
        str.push_str("🌪️");
    }
    if let Some(unknown) = features.iter().next() {
        str.push_str(&"❔".repeat(unknown.bits().count_ones() as usize));
    }
    format!("{} {}", str, sensor.name)
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

pub async fn sensors(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let sensors = match current_user {
        Some(_) => into_api_err(conn.get_sensors().await, StatusCode::INTERNAL_SERVER_ERROR)?,
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
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let sensor = into_api_err(
        conn.get_sensor(&host).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let Some(sensor) = sensor else {
        return api_err("Sensor not found", StatusCode::NOT_FOUND);
    };
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
    Extension(pool): Extension<DbPool>,
    form: RawForm,
) -> Result<Html<String>, ApiErrorResponse> {
    let sensor = match serde_urlencoded::from_bytes::<SensorFormData>(&form.0) {
        Ok(sensor) => sensor,
        Err(e) => {
            return api_err(
                format!("Form decoding error: {}", e),
                StatusCode::BAD_REQUEST,
            );
        }
    };
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    if let None = current_user {
        return api_err("Unauthorized", StatusCode::UNAUTHORIZED);
    }
    let host = host.replace('-', ".");
    let sensor_entity = into_api_err(
        conn.get_sensor(&host).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let Some(sensor_entity) = sensor_entity else {
        return api_err("Sensor not found", StatusCode::NOT_FOUND);
    };
    let Some(pair_id) = &sensor_entity.pair_id else {
        return api_err("Sensor not found", StatusCode::NOT_FOUND);
    };
    let sensor = into_api_err(
        Client::new().update_sensor(&host, pair_id, sensor).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    into_api_err(
        conn.update_sensor(&host, sensor).await,
        StatusCode::INTERNAL_SERVER_ERROR,
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
    Path(host): Path<String>,
    Extension(pool): Extension<DbPool>,
) -> Result<Html<String>, ApiErrorResponse> {
    let host = host.replace('-', ".");
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let affected = into_api_err(
        conn.delete_sensor(&host).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    if affected == 0 {
        return api_err("Sensor not found", StatusCode::NOT_FOUND);
    }

    Ok(Html(
        SensorsInnerTemplate {
            sensors: into_api_err(conn.get_sensors().await, StatusCode::INTERNAL_SERVER_ERROR)?,
            action_type: SensorActions::Overview,
        }
        .render()
        .unwrap(),
    ))
}
