use crate::{
    api_err,
    database::{areas::AreaDatabase, sensors::SensorDatabase, temp_data::TempDataDatabase},
    into_api_err,
    models::{
        db::{AreaEntity, SensorEntity, SensorFeatures},
        json::AreaFormData,
        Area, RequestData, User,
    },
    ApiErrorResponse,
};
use askama::Template;
use axum::{
    extract::{Path, Query},
    response::Html,
    Form,
};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/areas.html")]
pub struct AreasTemplate {
    pub current_user: Option<User>,
    pub areas: Vec<Area>,
}

#[derive(Template)]
#[template(path = "pages/areas-inner.html")]
pub struct AreasInnerTemplate {
    pub areas: Vec<Area>,
}

#[derive(Template)]
#[template(path = "components/area.html")]
pub struct AreaTemplate {
    pub area: Area,
}

#[derive(Template)]
#[template(path = "components/area-chart.html")]
pub struct AreaChartTemplate {
    pub area: Area,
    pub feature: String,
    pub sensor: Option<SensorEntity>,
    pub last: usize,
    pub no_control: bool,
    pub chart: Chart,
}

pub struct Chart {
    pub _type: String,
    pub labels: String,
    pub data: Vec<ChartData>,
}

pub struct ChartData {
    pub label: String,
    pub data: String,
    pub color: String,
    pub grid_color: String,
    pub min: i64,
    pub max: i64,
}

pub async fn areas(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    let areas = into_api_err(
        req_data.conn.get_areas().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    if req_data.is_hx_request {
        return Ok(Html(AreasInnerTemplate { areas }.render().unwrap()));
    }

    Ok(Html(
        AreasTemplate {
            current_user: req_data.user,
            areas,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn create_area(
    req_data: RequestData,
    Form(area_form): Form<AreaFormData>,
) -> Result<Html<String>, ApiErrorResponse> {
    if area_form.name.is_empty() {
        return api_err(
            "Area name cannot be empty",
            StatusCode::BAD_REQUEST,
            &req_data,
        );
    }
    into_api_err(
        req_data
            .conn
            .create_area(AreaEntity {
                id: 0,
                name: area_form.name,
            })
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let areas = into_api_err(
        req_data.conn.get_areas().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    if req_data.is_hx_request {
        return Ok(Html(AreasInnerTemplate { areas }.render().unwrap()));
    }

    Ok(Html(
        AreasTemplate {
            current_user: req_data.user,
            areas,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn update_area(
    Path(id): Path<i64>,
    req_data: RequestData,
    Form(area_form): Form<AreaFormData>,
) -> Result<Html<String>, ApiErrorResponse> {
    if area_form.name.is_empty() {
        return api_err(
            "Area name cannot be empty",
            StatusCode::BAD_REQUEST,
            &req_data,
        );
    }
    into_api_err(
        req_data
            .conn
            .update_area(AreaEntity {
                id,
                name: area_form.name,
            })
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let area = into_api_err(
        req_data.conn.get_area(id).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    Ok(Html(AreaTemplate { area }.render().unwrap()))
}

pub async fn delete_area(
    Path(id): Path<i64>,
    req_data: RequestData,
) -> Result<Html<String>, ApiErrorResponse> {
    into_api_err(
        req_data.conn.delete_area(id).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let areas = into_api_err(
        req_data.conn.get_areas().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    if req_data.is_hx_request {
        return Ok(Html(AreasInnerTemplate { areas }.render().unwrap()));
    }

    Ok(Html(
        AreasTemplate {
            current_user: req_data.user,
            areas,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Deserialize)]
pub struct AreaChartQuery {
    feature: Option<String>,
    last: Option<usize>,
    sensor: Option<String>,
    #[serde(rename = "no-control")]
    no_control: Option<String>,
}

pub async fn area_chart(
    Path(id): Path<i64>,
    req_data: RequestData,
    Query(feature): Query<AreaChartQuery>,
) -> Result<Html<String>, ApiErrorResponse> {
    let AreaChartQuery {
        feature,
        last,
        sensor,
        no_control,
    } = feature;
    match feature.as_deref() {
        Some("temp") => {
            let area = into_api_err(
                req_data.conn.get_area(id).await,
                StatusCode::INTERNAL_SERVER_ERROR,
                &req_data,
            )?;
            let sensor = match sensor {
                Some(host) => into_api_err(
                    req_data
                        .conn
                        .get_sensor(host.replace('-', ".").trim())
                        .await,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &req_data,
                )?,
                None => None,
            };
            let last = last.unwrap_or(1);
            let mut temp_data = into_api_err(
                req_data
                    .conn
                    .get_temp_data(
                        sensor.as_ref().map(|s| vec![s.host.clone()]).or_else(|| {
                            Some(area.sensors.iter().map(|s| s.host.clone()).collect())
                        }),
                        None,
                        None,
                        Some(
                            chrono::offset::Utc::now()
                                .checked_sub_days(chrono::Days::new(last as u64))
                                .unwrap()
                                .timestamp(),
                        ),
                    )
                    .await,
                StatusCode::INTERNAL_SERVER_ERROR,
                &req_data,
            )?;
            temp_data.reverse();
            let chart = Chart {
                _type: "line".to_string(),
                labels: format!(
                    "[{}]",
                    temp_data
                        .iter()
                        .map(|t| format!("new Date({} * 1000).toLocaleString()", t.timestamp))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                data: vec![
                    {
                        let temps = temp_data.iter().map(|t| t.temperature);
                        let min = temps
                            .clone()
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or_default();
                        let max = temps
                            .clone()
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or_default();
                        let margin = ((max - min) * 0.1).max(1.0);
                        let min = min - margin;
                        let max = max + margin;
                        ChartData {
                            label: "Temperature".to_string(),
                            data: format!(
                                "[{}]",
                                temps
                                    .map(|t| t.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            ),
                            color: "rgba(255, 99, 132, 0.9)".to_string(),
                            grid_color: "rgba(255, 99, 132, 0.4)".to_string(),
                            min: min as i64,
                            max: max as i64,
                        }
                    },
                    {
                        let hums = temp_data.iter().map(|t| t.humidity);
                        let min = hums
                            .clone()
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or_default();
                        let max = hums
                            .clone()
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or_default();
                        let margin = ((max - min) * 0.1).max(1.0);
                        let min = min - margin;
                        let max = max + margin;
                        ChartData {
                            label: "Humidity".to_string(),
                            data: format!(
                                "[{}]",
                                temp_data
                                    .iter()
                                    .map(|t| t.humidity.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            ),
                            color: "rgba(54, 162, 235, 0.9)".to_string(),
                            grid_color: "rgba(54, 162, 235, 0.4)".to_string(),
                            min: min as i64,
                            max: max as i64,
                        }
                    },
                ],
            };
            Ok(Html(
                AreaChartTemplate {
                    area,
                    feature: "temp".to_string(),
                    chart,
                    last,
                    no_control: no_control.is_some(),
                    sensor,
                }
                .render()
                .unwrap(),
            ))
        }
        _ => api_err("Invalid feature", StatusCode::BAD_REQUEST, &req_data),
    }
}
