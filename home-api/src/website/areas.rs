use crate::{
    api_err,
    database::areas::AreaDatabase,
    into_api_err,
    models::{
        db::{AreaEntity, SensorFeatures},
        json::AreaFormData,
        Area, RequestData, User,
    },
    ApiErrorResponse,
};
use askama::Template;
use axum::{extract::Path, response::Html, Form};
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

#[derive(Template)]
#[template(path = "components/area.html")]
pub struct AreaTemplate {
    pub area: Area,
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
