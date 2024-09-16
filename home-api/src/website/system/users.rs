use crate::{
    api_err,
    database::users::UserDatabase,
    into_api_err,
    models::{RequestData, User},
    ApiErrorResponse,
};
use askama::Template;
use axum::{extract::Path, response::Html, Form};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "pages/users.html")]
pub struct UsersTemplate {
    pub current_user: Option<User>,
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "pages/users-inner.html")]
pub struct UsersInnerTemplate {
    pub current_user: Option<User>,
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "components/user-row.html")]
pub struct UserRowTemplate {
    pub current_user: Option<User>,
    pub user: User,
}

pub async fn users(req_data: RequestData) -> Result<Html<String>, ApiErrorResponse> {
    let users = into_api_err(
        req_data.conn.get_users().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?
    .into_iter()
    .map(|u| u.into())
    .collect();

    if req_data.is_hx_request {
        return Ok(Html(
            UsersInnerTemplate {
                current_user: req_data.user,
                users,
            }
            .render()
            .unwrap(),
        ));
    }

    Ok(Html(
        UsersTemplate {
            current_user: req_data.user,
            users,
        }
        .render()
        .unwrap(),
    ))
}

#[derive(Deserialize)]
pub struct UserForm {
    name: String,
    password: String,
    confirm: String,
}

impl UserForm {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if self.password.is_empty() || self.confirm.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        if self.password != self.confirm {
            return Err("Passwords do not match".to_string());
        }

        Ok(())
    }
}

pub async fn create_user(
    req_data: RequestData,
    Form(form): Form<UserForm>,
) -> Result<Html<String>, ApiErrorResponse> {
    into_api_err(form.validate(), StatusCode::BAD_REQUEST, &req_data)?;
    let _user = into_api_err(
        req_data.conn.create_user(form.name, form.password).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;

    let users = into_api_err(
        req_data.conn.get_users().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?
    .into_iter()
    .map(|u| u.into())
    .collect();

    if req_data.is_hx_request {
        return Ok(Html(
            UsersInnerTemplate {
                current_user: req_data.user,
                users,
            }
            .render()
            .unwrap(),
        ));
    }

    Ok(Html(
        UsersTemplate {
            current_user: req_data.user,
            users,
        }
        .render()
        .unwrap(),
    ))
}

pub async fn change_password(
    req_data: RequestData,
    Form(form): Form<UserForm>,
) -> Result<Html<String>, ApiErrorResponse> {
    into_api_err(form.validate(), StatusCode::BAD_REQUEST, &req_data)?;
    into_api_err(
        req_data
            .conn
            .change_password(&form.name, form.password)
            .await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;

    let Some(user) = into_api_err(
        req_data.conn.get_user(&form.name).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?
    else {
        return api_err("User not found", StatusCode::NOT_FOUND, &req_data);
    };

    if req_data.is_hx_request {
        return Ok(Html(
            UserRowTemplate {
                current_user: req_data.user,
                user: user.into(),
            }
            .render()
            .unwrap(),
        ));
    }

    Ok(Html(
        UserRowTemplate {
            current_user: req_data.user,
            user: user.into(),
        }
        .render()
        .unwrap(),
    ))
}

pub async fn delete_user(
    req_data: RequestData,
    Path(name): Path<String>,
) -> Result<Html<String>, ApiErrorResponse> {
    into_api_err(
        req_data.conn.delete_user(&name).await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?;

    let users = into_api_err(
        req_data.conn.get_users().await,
        StatusCode::INTERNAL_SERVER_ERROR,
        &req_data,
    )?
    .into_iter()
    .map(|u| u.into())
    .collect();

    if req_data.is_hx_request {
        return Ok(Html(
            UsersInnerTemplate {
                current_user: req_data.user,
                users,
            }
            .render()
            .unwrap(),
        ));
    }

    Ok(Html(
        UsersTemplate {
            current_user: req_data.user,
            users,
        }
        .render()
        .unwrap(),
    ))
}
