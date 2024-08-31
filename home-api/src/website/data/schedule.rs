use crate::{
    database::data_schedule::DataScheduleDatabase,
    into_api_err,
    models::{
        db::{DataScheduleEntry, SensorFeatures},
        json::ScheduleEntryFormData,
        RequestData, User,
    },
    services::sensor_data_service::SensorDataService,
    ApiErrorResponse,
};
use askama::Template;
use axum::{extract::Query, response::Html, Extension, Form};
use reqwest::StatusCode;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Template)]
#[template(path = "pages/data-schedule.html")]
pub struct DataScheduleTemplate {
    pub current_user: Option<User>,
    pub schedule: Vec<DataScheduleEntry>,
}

#[derive(Template)]
#[template(path = "pages/data-schedule-inner.html")]
pub struct DataScheduleInnerTemplate {
    pub schedule: Vec<DataScheduleEntry>,
}

pub fn delete_query(entry: &DataScheduleEntry) -> String {
    format!(
        "features={}&interval_ms={}",
        entry.features.bits(),
        entry.interval_ms
    )
}

pub async fn data_schedule(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    let schedule = into_api_err(
        req_data.conn.get_schedule().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;

    if req_data.is_hx_request {
        return Ok(Html(
            DataScheduleInnerTemplate { schedule }.render().unwrap(),
        ));
    }

    Ok(Html(
        DataScheduleTemplate {
            current_user: req_data.user,
            schedule,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn create_schedule_entry(
    req_data: RequestData,
    Extension(data_service): Extension<Arc<Mutex<SensorDataService>>>,
    Form(schedule_entry): Form<ScheduleEntryFormData>,
) -> Result<Html<String>, ApiErrorResponse> {
    let entry = into_api_err(
        schedule_entry.try_into(),
        StatusCode::BAD_REQUEST,
        &req_data,
    )?;
    let new_entry = into_api_err(
        req_data.conn.create_entry(entry).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    if new_entry.is_some() {
        _ = data_service.lock().await.restart().await;
    }
    let schedule = into_api_err(
        req_data.conn.get_schedule().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;

    Ok(Html(
        DataScheduleInnerTemplate { schedule }.render().unwrap(),
    ))
}

pub async fn delete_schedule_entry(
    req_data: RequestData,
    Query(query): Query<BTreeMap<String, String>>,
    Extension(data_service): Extension<Arc<Mutex<SensorDataService>>>,
) -> Result<Html<String>, ApiErrorResponse> {
    let entry = into_api_err(
        query
            .get("features")
            .ok_or(anyhow::anyhow!("Missing `features` field"))
            .and_then(|f| Ok(f.parse()?))
            .map(SensorFeatures::from_bits_retain)
            .and_then(|f| {
                query
                    .get("interval_ms")
                    .ok_or(anyhow::anyhow!("Missing `interval_ms` field"))
                    .and_then(|i| Ok((f, i.parse()?)))
                    .map(|(f, i)| DataScheduleEntry {
                        features: f,
                        interval_ms: i,
                    })
            }),
        StatusCode::BAD_REQUEST,
        &req_data,
    )?;
    let success = into_api_err(
        req_data.conn.delete_entry(entry).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;

    if success {
        _ = data_service.lock().await.restart().await;
    }

    let schedule = into_api_err(
        req_data.conn.get_schedule().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;

    Ok(Html(
        DataScheduleInnerTemplate { schedule }.render().unwrap(),
    ))
}
