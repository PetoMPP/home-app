use crate::{
    database::{DbPool, UserDatabase},
    models::auth::Token,
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
    pub error: Option<String>,
}

#[derive(Template, Default)]
#[template(path = "pages/login-inner.html")]
pub struct LoginInnerTemplate {
    pub error: Option<String>,
}

pub async fn login_page(headers: HeaderMap) -> Html<String> {
    match headers.contains_key("Hx-Request") {
        true => Html(LoginInnerTemplate::default().render().unwrap()),
        false => Html(LoginTemplate::default().render().unwrap()),
    }
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
        header_map.insert(SET_COOKIE, format!("session={}", token).parse().unwrap());
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
