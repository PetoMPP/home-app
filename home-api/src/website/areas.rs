use super::is_hx_request;
use crate::{
    database::DbPool,
    into_api_err,
    models::{
        auth::Token,
        db::{SensorEntity, SensorFeatures},
        Area, User,
    },
    ApiErrorResponse,
};
use askama::Template;
use axum::{http::HeaderMap, response::Html, Extension};
use reqwest::StatusCode;

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

pub async fn areas(
    Extension(pool): Extension<DbPool>,
    token: Option<Token>,
    headers: HeaderMap,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let areas = vec![
        Area {
            id: 1,
            name: "Area 1".to_string(),
            sensors: vec![
                SensorEntity {
                    name: "Sensor 1".to_string(),
                    area: None,
                    features: SensorFeatures::TEMPERATURE | SensorFeatures::MOTION,
                    host: "11.44.21.1".to_string(),
                    pair_id: None,
                },
                SensorEntity {
                    name: "Sensor 2".to_string(),
                    area: None,
                    features: SensorFeatures::TEMPERATURE | SensorFeatures::MOTION,
                    host: "12.44.21.1".to_string(),
                    pair_id: None,
                },
            ],
        },
        Area {
            id: 2,
            name: "Area 2".to_string(),
            sensors: vec![
                SensorEntity {
                    name: "Sensor 3".to_string(),
                    area: None,
                    features: SensorFeatures::TEMPERATURE | SensorFeatures::MOTION,
                    host: "13.44.21.1".to_string(),
                    pair_id: None,
                },
                SensorEntity {
                    name: "Sensor 4".to_string(),
                    area: None,
                    features: SensorFeatures::TEMPERATURE,
                    host: "4+656+54".to_string(),
                    pair_id: None,
                },
                SensorEntity {
                    name: "Sensor 5".to_string(),
                    area: None,
                    features: SensorFeatures::from_bits_retain(0b11111),
                    host: "4+656+54".to_string(),
                    pair_id: None,
                },
            ],
        },
    ];

    if is_hx_request(&headers) {
        return Ok(Html(AreasInnerTemplate { areas }.render().unwrap()));
    }

    Ok(Html(
        AreasTemplate {
            current_user: current_user.clone(),
            areas,
        }
        .render()
        .unwrap(),
    ))
}
