use crate::{
    api_err,
    database::{user_sessions::UserSessionDatabase, users::UserDatabase},
    into_api_err,
    models::{
        auth::{Claims, Token},
        NormalizedString, RequestData, User,
    },
    ApiErrorResponse,
};
use askama::Template;
use axum::{http::HeaderMap, response::Html, Form};
use reqwest::{header::SET_COOKIE, StatusCode};
use serde::Deserialize;

#[derive(Template, Default)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate {
    pub current_user: Option<User>,
}

#[derive(Template, Default)]
#[template(path = "pages/login-inner.html")]
pub struct LoginInnerTemplate;

pub async fn login_page(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    if req_data.is_hx_request {
        return Ok(Html(LoginInnerTemplate.render().unwrap()));
    }
    Ok(Html(
        LoginTemplate {
            current_user: req_data.user,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

pub async fn login(
    req_data: RequestData,
    Form(credentials): Form<Credentials>,
) -> Result<(StatusCode, HeaderMap), ApiErrorResponse> {
    let user = into_api_err(
        req_data.conn.get_user(&credentials.username).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let Some(user) = user else {
        return api_err(
            "Invalid username or password",
            StatusCode::UNAUTHORIZED,
            &req_data,
        );
    };
    if !user.password.verify(&credentials.password) {
        return api_err(
            "Invalid username or password",
            StatusCode::UNAUTHORIZED,
            &req_data,
        );
    }

    let token = into_api_err(
        Token::new(&user),
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    into_api_err(
        req_data
            .conn
            .create_session(user.normalized_name.clone(), token.clone())
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let mut header_map = HeaderMap::new();
    header_map.insert(SET_COOKIE, format!("session={}", *token).parse().unwrap());
    header_map.insert("HX-Redirect", "/".parse().unwrap());
    Ok((StatusCode::OK, header_map))
}

pub async fn logout(req_data: RequestData) -> Result<(StatusCode, HeaderMap), ApiErrorResponse> {
    let Some(token) = &req_data.token else {
        return api_err("No session cookie", StatusCode::UNAUTHORIZED, &req_data);
    };
    let Ok(claims): Result<Claims, _> = token.try_into() else {
        return api_err("Invalid session", StatusCode::UNAUTHORIZED, &req_data);
    };
    into_api_err(
        req_data
            .conn
            .delete_session(NormalizedString::new(claims.sub), token.clone())
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;
    let mut header_map = HeaderMap::new();
    header_map.insert(SET_COOKIE, "session=;".parse().unwrap());
    header_map.insert("HX-Redirect", "/".parse().unwrap());
    Ok((StatusCode::OK, header_map))
}
