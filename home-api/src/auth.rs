use crate::{
    database::{user_sessions::UserSessionDatabase, DbPool},
    models::{
        auth::{Claims, Token},
        NormalizedString, RequestData,
    },
};
use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use reqwest::{header, StatusCode};

pub async fn validate_user_session(
    req_data: RequestData,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, HeaderMap)> {
    let error = || {
        let mut header_map = HeaderMap::new();
        match req_data.is_hx_request {
            true => {
                header_map.insert("HX-Redirect", "/login".parse().unwrap());

                (StatusCode::UNAUTHORIZED, header_map)
            }
            false => {
                header_map.insert(header::LOCATION, "/login".parse().unwrap());
                (StatusCode::SEE_OTHER, header_map)
            }
        }
    };
    let (claims, token) = Token::try_from(&req_data.headers)
        .and_then(|t| (&t).try_into().map(|c: Claims| (c, t)))
        .map_err(|_| error())?;
    let normalized_name = NormalizedString::new(&claims.sub);
    let _session = req_data
        .conn
        .get_session(normalized_name.clone(), token.clone())
        .await
        .ok()
        .and_then(|s| s)
        .ok_or_else(error)?;
    if !claims.validate() {
        req_data
            .conn
            .delete_session(normalized_name, token)
            .await
            .ok();
        return Err(error());
    }

    Ok(next.run(request).await)
}

pub fn start_user_session_watchdog(pool: DbPool) {
    tokio::spawn(async move {
        loop {
            _ = delete_tokens(&pool).await;
        }
    });
}

async fn delete_tokens(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    let conn = pool.get().await?;
    let sessions = conn.get_sessions().await?;
    let mut invalid_sessions = vec![];
    for session in sessions {
        let Ok(claims): Result<Claims, _> = (&session.token).try_into() else {
            invalid_sessions.push(session);
            continue;
        };
        if !claims.validate() {
            invalid_sessions.push(session);
        }
    }
    if !invalid_sessions.is_empty() {
        return conn.delete_sessions(invalid_sessions).await.map(|_| ());
    }

    Ok(())
}
