use crate::{
    models::{
        db::{SensorEntity, SensorFeatures},
        Area, RequestData, User,
    },
    ApiErrorResponse,
};
use askama::Template;
use axum::response::Html;

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

pub async fn areas(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
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
