use crate::{
    api_err,
    database::{areas::AreaDatabase, sensors::SensorDatabase},
    into_api_err,
    models::{
        db::{AreaEntity, SensorEntity, SensorFeatures},
        json::SensorFormData,
        RequestData, User,
    },
    services::sensor_service::SensorService,
    ApiErrorResponse,
};
use askama::Template;
use axum::{extract::Path, http::HeaderMap, response::Html, Form};
use reqwest::{Client, StatusCode};

#[derive(Template)]
#[template(path = "pages/sensors.html")]
pub struct SensorsTemplate {
    pub current_user: Option<User>,
    pub sensors: Vec<SensorEntity>,
    pub action_type: SensorActions,
    pub areas: Vec<AreaEntity>,
}

#[derive(Template)]
#[template(path = "pages/sensors-inner.html")]
pub struct SensorsInnerTemplate {
    pub sensors: Vec<SensorEntity>,
    pub action_type: SensorActions,
    pub areas: Vec<AreaEntity>,
}

#[derive(Template)]
#[template(path = "components/sensor.html")]
pub struct SensorTemplate {
    pub sensor: SensorEntity,
    pub action_type: SensorActions,
    pub areas: Vec<(AreaEntity, bool)>,
}

pub fn areas(areas: &Vec<AreaEntity>, sensor: &SensorEntity) -> Vec<(AreaEntity, bool)> {
    areas
        .iter()
        .cloned()
        .map(|a| {
            let id = a.id;
            (a, sensor.area.as_ref().map(|a| a.id) == Some(id))
        })
        .collect()
}

pub fn areas_empty() -> Vec<(AreaEntity, bool)> {
    Vec::new()
}

pub fn sensor_style(sensor: &SensorEntity) -> &'static str {
    if sensor.features.is_empty() {
        return "bg-neutral border-neutral-content text-neutral-content opacity-70";
    }

    "bg-base-300 border-base-content text-base-content"
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
    let areas = into_api_err(
        req_data.conn.get_area_entities().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    match req_data.is_hx_request {
        true => Ok(Html(
            SensorsInnerTemplate {
                sensors,
                action_type: SensorActions::Overview,
                areas,
            }
            .render()
            .unwrap(),
        )),
        false => Ok(Html(
            SensorsTemplate {
                sensors,
                current_user: req_data.user,
                action_type: SensorActions::Overview,
                areas,
            }
            .render()
            .unwrap(),
        )),
    }
}

pub async fn update_sensor(
    req_data: RequestData,
    Path(host): Path<String>,
    Form(sensor): Form<SensorFormData>,
) -> Result<Html<String>, ApiErrorResponse> {
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
    let sensor_response = into_api_err(
        Client::new()
            .update_sensor(&host, pair_id, sensor.clone())
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    into_api_err(
        req_data
            .conn
            .update_sensor(
                &host,
                SensorEntity {
                    host: host.clone(),
                    pair_id: None,
                    name: sensor_response.name.clone(),
                    area: sensor.area_id.clone().parse().ok().map(|id| AreaEntity {
                        id,
                        name: String::new(),
                    }),
                    features: SensorFeatures::from_bits_retain(sensor_response.features),
                },
            )
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let sensor_entity = into_api_err(
        req_data
            .conn
            .get_sensor(&host)
            .await
            .and_then(|s| Ok(s.ok_or(anyhow::anyhow!("Sensor not found"))?)),
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let areas = areas(
        &into_api_err(
            req_data.conn.get_area_entities().await,
            StatusCode::INTERNAL_SERVER_ERROR,
            &req_data,
        )?,
        &sensor_entity,
    );
    Ok(Html(
        SensorTemplate {
            sensor: sensor_entity,
            action_type: SensorActions::Overview,
            areas,
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

    let areas = into_api_err(
        req_data.conn.get_area_entities().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;

    Ok(Html(
        SensorsInnerTemplate {
            sensors: into_api_err(
                req_data.conn.get_sensors().await,
                StatusCode::INTERNAL_SERVER_ERROR,
                &req_data,
            )?,
            action_type: SensorActions::Overview,
            areas,
        }
        .render()
        .unwrap(),
    ))
}
