use super::{Database, DbConn};
use crate::models::{auth::Password, db::UserEntity, NormalizedString};

pub trait UserDatabase {
    async fn get_user(
        &self,
        username: &str,
    ) -> Result<Option<UserEntity>, Box<dyn std::error::Error>>;
    async fn get_users(&self) -> Result<Vec<UserEntity>, Box<dyn std::error::Error>>;
    async fn create_user(
        &self,
        name: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<UserEntity, Box<dyn std::error::Error>>;
    async fn ensure_admin(&self) -> Result<Option<UserEntity>, Box<dyn std::error::Error>>;
}

impl UserDatabase for DbConn {
    async fn get_user(
        &self,
        username: &str,
    ) -> Result<Option<UserEntity>, Box<dyn std::error::Error>> {
        let username = NormalizedString::new(username);
        self.query_single::<UserEntity>(&format!(
            "SELECT * FROM users WHERE normalized_name = '{}' LIMIT 1",
            *username
        ))
        .await
    }

    async fn create_user(
        &self,
        name: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<UserEntity, Box<dyn std::error::Error>> {
        let name = name.into();
        let normalized_name = NormalizedString::new(&name);
        let password = Password::new(password.into());
        Ok(self.query_single::<UserEntity>(&format!(
            "INSERT INTO users (name, normalized_name, password) VALUES ('{}', '{}', '{}') RETURNING *",
            &name, *normalized_name, password
        ))
        .await?.ok_or("Failed to create user")?)
    }

    async fn ensure_admin(&self) -> Result<Option<UserEntity>, Box<dyn std::error::Error>> {
        let users = self.get_users().await?;
        if users.is_empty() {
            return Ok(Some(self.create_user("admin", "admin").await?));
        }
        Ok(None)
    }

    async fn get_users(&self) -> Result<Vec<UserEntity>, Box<dyn std::error::Error>> {
        self.query::<UserEntity>("SELECT * FROM users").await
    }
}
