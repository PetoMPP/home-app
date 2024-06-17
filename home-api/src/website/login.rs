use crate::{
    database::{user_sessions::UserSessionDatabase, users::UserDatabase, DbPool},
    into_err,
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
    pub alert_message: Option<String>,
}

#[derive(Template, Default)]
#[template(path = "pages/login-inner.html")]
pub struct LoginInnerTemplate {
    pub alert_message: Option<String>,
}

pub async fn login_page(
    token: Option<Token>,
    Extension(pool): Extension<DbPool>,
) -> Result<Html<String>, ApiErrorResponse> {
    let conn = pool.get().await.map_err(into_err)?;
    let current_user = Token::get_valid_user(token, &conn)
        .await
        .map_err(into_err)?;
    Ok(Html(
        LoginTemplate {
            current_user: current_user.clone(),
            ..Default::default()
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
) -> Result<(StatusCode, HeaderMap), (StatusCode, Html<String>)> {
    let conn = pool.get().await.unwrap();
    let user = conn
        .get_user(&credentials.username)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    LoginInnerTemplate {
                        alert_message: Some(e.to_string()),
                    }
                    .render()
                    .unwrap(),
                ),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Html(
                LoginInnerTemplate {
                    alert_message: Some("Invalid username or password".to_string()),
                }
                .render()
                .unwrap(),
            ),
        ))?;

    if user.password.verify(&credentials.password) {
        let mut header_map = HeaderMap::new();
        let token = Token::new(&user).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    LoginInnerTemplate {
                        alert_message: Some(e.to_string()),
                    }
                    .render()
                    .unwrap(),
                ),
            )
        })?;
        conn.create_session(user.normalized_name.clone(), token.clone())
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(
                        LoginInnerTemplate {
                            alert_message: Some("Failed to create session".to_string()),
                        }
                        .render()
                        .unwrap(),
                    ),
                )
            })?;
        header_map.insert(SET_COOKIE, format!("session={}", *token).parse().unwrap());
        header_map.insert("HX-Redirect", "/".parse().unwrap());
        Ok((StatusCode::OK, header_map))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Html(
                LoginInnerTemplate {
                    alert_message: Some("Invalid username or password".to_string()),
                }
                .render()
                .unwrap(),
            ),
        ))
    }
}

pub async fn logout(
    Extension(pool): Extension<DbPool>,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap), (StatusCode, Html<String>)> {
    let token = Token::try_from(&headers).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Html(
                LoginInnerTemplate {
                    alert_message: Some("No session cookie".to_string()),
                }
                .render()
                .unwrap(),
            ),
        )
    })?;
    let claims: Claims = (&token).try_into().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Html(
                LoginInnerTemplate {
                    alert_message: Some("Invalid session".to_string()),
                }
                .render()
                .unwrap(),
            ),
        )
    })?;
    let conn = pool.get().await.unwrap();
    conn.delete_session(NormalizedString::new(claims.sub), token)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    LoginInnerTemplate {
                        alert_message: Some("Failed to delete session".to_string()),
                    }
                    .render()
                    .unwrap(),
                ),
            )
        })?;
    let mut header_map = HeaderMap::new();
    header_map.insert(SET_COOKIE, "session=;".parse().unwrap());
    header_map.insert("HX-Redirect", "/".parse().unwrap());
    Ok((StatusCode::OK, header_map))
}
