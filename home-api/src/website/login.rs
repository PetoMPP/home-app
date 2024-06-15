use crate::{
    database::{user_sessions::UserSessionDatabase, users::UserDatabase, DbPool},
    models::{
        auth::{Claims, Token},
        NormalizedString, User,
    },
};
use askama::Template;
use axum::{http::HeaderMap, response::Html, Extension, Form};
use reqwest::{
    header::{LOCATION, SET_COOKIE},
    StatusCode,
};
use serde::Deserialize;

#[derive(Template, Default)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate {
    pub current_user: Option<User>,
    pub error: Option<String>,
}

#[derive(Template, Default)]
#[template(path = "pages/login-inner.html")]
pub struct LoginInnerTemplate {
    pub error: Option<String>,
}

pub async fn login_page(headers: HeaderMap, current_user: Option<User>) -> Html<String> {
    let login_tmpl = match headers.contains_key("Hx-Request") {
        true => LoginInnerTemplate::default().render().unwrap(),
        false => LoginTemplate {
            current_user: current_user.clone(),
            ..Default::default()
        }
        .render()
        .unwrap(),
    };

    Html(login_tmpl)
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
                        error: Some(e.to_string()),
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
                    error: Some("Invalid username or password".to_string()),
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
                        error: Some(e.to_string()),
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
                            error: Some("Failed to create session".to_string()),
                        }
                        .render()
                        .unwrap(),
                    ),
                )
            })?;
        header_map.insert(SET_COOKIE, format!("session={}", *token).parse().unwrap());
        header_map.insert(LOCATION, "/".parse().unwrap());
        Ok((StatusCode::SEE_OTHER, header_map))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Html(
                LoginInnerTemplate {
                    error: Some("Invalid username or password".to_string()),
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
                    error: Some("No session cookie".to_string()),
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
                    error: Some("Invalid session".to_string()),
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
                        error: Some("Failed to delete session".to_string()),
                    }
                    .render()
                    .unwrap(),
                ),
            )
        })?;
    let mut header_map = HeaderMap::new();
    header_map.insert(SET_COOKIE, "session=;".parse().unwrap());
    header_map.insert(LOCATION, "/".parse().unwrap());
    Ok((StatusCode::SEE_OTHER, header_map))
}
