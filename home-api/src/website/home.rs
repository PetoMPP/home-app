use crate::{
    api_error::into_api_err,
    api_error::ApiErrorResponse,
    database::{areas::AreaDatabase, temp_data::TempDataDatabase},
    models::{Area, RequestData, User},
};
use askama::Template;
use axum::response::Html;
use reqwest::StatusCode;

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub current_user: Option<User>,
    pub areas: Vec<(Area, f32, f32)>,
}

#[derive(Template)]
#[template(path = "pages/home-inner.html")]
pub struct HomeInnerTemplate {
    pub areas: Vec<(Area, f32, f32)>,
}

pub async fn home(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    let areas = into_api_err(
        req_data.conn.get_areas().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let mut areas_full = vec![];
    for area in areas {
        let (temp, hum) = into_api_err(
            req_data
                .conn
                .get_temp_data(
                    Some(area.sensors.iter().map(|s| s.host.clone()).collect()),
                    Some(1),
                    None,
                    None,
                )
                .await,
            StatusCode::INTERNAL_SERVER_ERROR,
            &req_data,
        )?
        .first()
        .map(|t| (t.temperature, t.humidity))
        .unwrap_or_default();

        areas_full.push((area, temp, hum));
    }

    if req_data.is_hx_request {
        return Ok(Html(
            HomeInnerTemplate { areas: areas_full }.render().unwrap(),
        ));
    }

    Ok(Html(
        HomeTemplate {
            current_user: req_data.user,
            areas: areas_full,
        }
        .render()
        .unwrap(),
    ))
}
