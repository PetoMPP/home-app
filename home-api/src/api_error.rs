use crate::{
    models::RequestData,
    website::{components::alert::AlertTemplate, ErrorTemplate},
};
use askama::Template;
use axum::http::HeaderMap;
use reqwest::StatusCode;
use std::fmt::Display;

pub type ApiErrorResponse = (StatusCode, HeaderMap, axum::response::Html<String>);

pub fn api_err<T>(
    error: impl Into<String>,
    code: StatusCode,
    req_data: &RequestData,
) -> Result<T, ApiErrorResponse> {
    if req_data.is_hx_request {
        return Err((
            code,
            error_headers(),
            axum::response::Html(
                AlertTemplate {
                    alert_message: Some(error.into()),
                    alert_type: Some(code.into()),
                    swap_oob: false,
                }
                .render()
                .unwrap(),
            ),
        ));
    }

    Err((
        code,
        error_headers(),
        axum::response::Html(
            ErrorTemplate {
                current_user: req_data.user.clone(),
                status: code,
                message: error.into(),
            }
            .render()
            .unwrap(),
        ),
    ))
}

pub fn into_api_err<T>(
    result: Result<T, impl Display>,
    code: StatusCode,
    req_data: &RequestData,
) -> Result<T, ApiErrorResponse> {
    if req_data.is_hx_request {
        return result.map_err(|e| {
            (
                code,
                error_headers(),
                axum::response::Html(
                    AlertTemplate {
                        alert_message: Some(e.to_string()),
                        alert_type: Some(code.into()),
                        swap_oob: false,
                    }
                    .render()
                    .unwrap(),
                ),
            )
        });
    }

    result.map_err(|e| {
        (
            code,
            error_headers(),
            axum::response::Html(
                ErrorTemplate {
                    current_user: req_data.user.clone(),
                    status: code,
                    message: e.to_string(),
                }
                .render()
                .unwrap(),
            ),
        )
    })
}

fn error_headers() -> HeaderMap {
    let mut header_map = HeaderMap::new();
    header_map.insert("Hx-Retarget", "#alert-element".parse().unwrap());
    header_map.insert("Hx-Reswap", "outerHTML".parse().unwrap());
    header_map
}
