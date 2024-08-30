use std::{collections::BTreeMap, sync::Arc};

use crate::{
    database::{data_schedule::DataScheduleDatabase, DbPool},
    into_api_err,
    models::{
        auth::Token,
        db::{DataScheduleEntry, SensorFeatures},
        json::ScheduleEntryFormData,
        User,
    },
    services::sensor_data_service::SensorDataService,
    website::is_hx_request,
    ApiErrorResponse,
};
use askama::Template;
use axum::{extract::Query, http::HeaderMap, response::Html, Extension, Form};
use reqwest::StatusCode;
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

pub async fn data_schedule(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;

    let schedule = into_api_err(conn.get_schedule().await, StatusCode::INTERNAL_SERVER_ERROR)?;

    if is_hx_request(&headers) {
        return Ok(Html(
            DataScheduleInnerTemplate { schedule }.render().unwrap(),
        ));
    }

    Ok(Html(
        DataScheduleTemplate {
            current_user: current_user.clone(),
            schedule,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn create_schedule_entry(
    Extension(pool): Extension<DbPool>,
    Extension(data_service): Extension<Arc<Mutex<SensorDataService>>>,
    Form(schedule_entry): Form<ScheduleEntryFormData>,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let entry = into_api_err(schedule_entry.try_into(), StatusCode::BAD_REQUEST)?;
    let new_entry = into_api_err(
        conn.create_entry(entry).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    if new_entry.is_some() {
        _ = data_service.lock().await.restart().await;
    }
    let schedule = into_api_err(conn.get_schedule().await, StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(
        DataScheduleInnerTemplate { schedule }.render().unwrap(),
    ))
}

pub async fn delete_schedule_entry(
    Query(query): Query<BTreeMap<String, String>>,
    Extension(pool): Extension<DbPool>,
    Extension(data_service): Extension<Arc<Mutex<SensorDataService>>>,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
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
    )?;
    let success = into_api_err(
        conn.delete_entry(entry).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;

    if success {
        _ = data_service.lock().await.restart().await;
    }

    let schedule = into_api_err(conn.get_schedule().await, StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(
        DataScheduleInnerTemplate { schedule }.render().unwrap(),
    ))
}
