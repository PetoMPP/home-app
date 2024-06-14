use askama::Template;
use axum::{http::HeaderMap, response::Html};

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate;

#[derive(Template)]
#[template(path = "pages/login-inner.html")]
pub struct LoginInnerTemplate;

pub async fn login(headers: HeaderMap) -> Html<String> {
    match headers.contains_key("Hx-Request") {
        true => Html(LoginInnerTemplate.render().unwrap()),
        false => Html(LoginTemplate.render().unwrap()),
    }
}
