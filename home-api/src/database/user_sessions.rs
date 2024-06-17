use crate::models::{auth::Token, db::UserSession, NormalizedString};

use super::{Database, DbConn};

pub trait UserSessionDatabase {
    async fn create_session(
        &self,
        normalized_name: NormalizedString,
        token: Token,
    ) -> Result<UserSession, Box<dyn std::error::Error>>;
    async fn get_session(
        &self,
        normalized_name: NormalizedString,
        token: Token,
    ) -> Result<Option<UserSession>, Box<dyn std::error::Error>>;
    async fn get_sessions(&self) -> Result<Vec<UserSession>, Box<dyn std::error::Error>>;
    async fn delete_session(
        &self,
        normalized_name: NormalizedString,
        token: Token,
    ) -> Result<bool, Box<dyn std::error::Error>>;
    async fn delete_sessions(
        &self,
        sessions: Vec<UserSession>,
    ) -> Result<usize, Box<dyn std::error::Error>>;
}

impl UserSessionDatabase for DbConn {
    async fn create_session(
        &self,
        normalized_name: NormalizedString,
        token: Token,
    ) -> Result<UserSession, Box<dyn std::error::Error>> {
        Ok(self
            .query_single::<UserSession>(&format!(
                "INSERT INTO user_sessions (normalized_name, token) VALUES ('{}', '{}') RETURNING *",
                *normalized_name, *token
            ))
            .await?
            .ok_or("Failed to create session")?)
    }

    async fn get_session(
        &self,
        normalized_name: NormalizedString,
        token: Token,
    ) -> Result<Option<UserSession>, Box<dyn std::error::Error>> {
        self.query_single::<UserSession>(&format!(
            "SELECT * FROM user_sessions WHERE normalized_name = '{}' AND token = '{}' LIMIT 1",
            *normalized_name, *token
        ))
        .await
    }

    async fn get_sessions(&self) -> Result<Vec<UserSession>, Box<dyn std::error::Error>> {
        self.query::<UserSession>("SELECT * FROM user_sessions")
            .await
    }

    async fn delete_session(
        &self,
        normalized_name: NormalizedString,
        token: Token,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self
            .execute(&format!(
                "DELETE FROM user_sessions WHERE normalized_name = '{}' AND token = '{}'",
                *normalized_name, *token
            ))
            .await?
            > 0)
    }

    async fn delete_sessions(
        &self,
        sessions: Vec<UserSession>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        self.execute(&format!(
            "DELETE FROM user_sessions WHERE {}",
            sessions
                .iter()
                .map(|session| {
                    format!(
                        "(normalized_name = '{}' AND token = '{}')",
                        *session.normalized_name, *session.token
                    )
                })
                .collect::<Vec<String>>()
                .join(" OR ")
        ))
        .await
    }
}
