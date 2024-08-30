use crate::{
    api_err,
    database::{user_sessions::UserSessionDatabase, users::UserDatabase, DbPool},
    into_api_err,
    models::{
        auth::{Claims, Token},
        NormalizedString, User,
    },
    ApiErrorResponse,
};
use askama::Template;
use axum::{http::HeaderMap, response::Html, Extension, Form};
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

pub async fn login_page(
    token: Option<Token>,
    Extension(pool): Extension<DbPool>,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_user = into_api_err(
        Token::get_valid_user(token, &conn).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;

    Ok(Html(
        LoginTemplate {
            current_user: current_user.clone(),
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
    Extension(pool): Extension<DbPool>,
    Form(credentials): Form<Credentials>,
) -> Result<(StatusCode, HeaderMap), ApiErrorResponse> {
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    let user = into_api_err(
        conn.get_user(&credentials.username).await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let Some(user) = user else {
        return api_err("Invalid username or password", StatusCode::UNAUTHORIZED);
    };
    if !user.password.verify(&credentials.password) {
        return api_err("Invalid username or password", StatusCode::UNAUTHORIZED);
    }

    let token = into_api_err(Token::new(&user), StatusCode::INTERNAL_SERVER_ERROR)?;
    into_api_err(
        conn.create_session(user.normalized_name.clone(), token.clone())
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let mut header_map = HeaderMap::new();
    header_map.insert(SET_COOKIE, format!("session={}", *token).parse().unwrap());
    header_map.insert("HX-Redirect", "/".parse().unwrap());
    Ok((StatusCode::OK, header_map))
}

pub async fn logout(
    Extension(pool): Extension<DbPool>,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap), ApiErrorResponse> {
    let Ok(token) = Token::try_from(&headers) else {
        return api_err("No session cookie", StatusCode::UNAUTHORIZED);
    };
    let Ok(claims): Result<Claims, _> = (&token).try_into() else {
        return api_err("Invalid session", StatusCode::UNAUTHORIZED);
    };
    let conn = into_api_err(pool.get().await, StatusCode::INTERNAL_SERVER_ERROR)?;
    into_api_err(
        conn.delete_session(NormalizedString::new(claims.sub), token)
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
    )?;
    let mut header_map = HeaderMap::new();
    header_map.insert(SET_COOKIE, "session=;".parse().unwrap());
    header_map.insert("HX-Redirect", "/".parse().unwrap());
    Ok((StatusCode::OK, header_map))
}
