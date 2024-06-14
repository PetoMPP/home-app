use home_common::models::Sensor;
use r2d2_sqlite::rusqlite::OptionalExtension;

use crate::models::{
    auth::Password,
    db::{SensorEntity, UserEntity},
    NormalizedString,
};

pub type DbManager = deadpool_r2d2::Manager<r2d2_sqlite::SqliteConnectionManager>;
pub type DbPool = deadpool_r2d2::Pool<DbManager>;
pub type DbConn = deadpool::managed::Object<DbManager>;

pub trait Database {
    async fn execute(&self, query: &str) -> Result<usize, Box<dyn std::error::Error>>;
    async fn query<T: FromRow + Send + Sync + 'static>(
        &self,
        query: &str,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>>;
    async fn query_single<T: FromRow + Send + Sync + 'static>(
        &self,
        query: &str,
    ) -> Result<Option<T>, Box<dyn std::error::Error>>;
}

pub trait FromRow: Sized {
    fn from_row(row: &r2d2_sqlite::rusqlite::Row) -> r2d2_sqlite::rusqlite::Result<Self>;
}

pub trait SensorDatabase {
    async fn get_sensors(&self) -> Result<Vec<Sensor>, Box<dyn std::error::Error>>;
}

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
    // fn create_user(&self, user: &User) -> Result<(), Error>;
}

impl Database for DbConn {
    async fn execute(&self, query: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let query = query.to_string();
        Ok(self
            .interact(move |conn| conn.execute(query.as_str(), []))
            .await??)
    }

    async fn query<T: FromRow + Send + Sync + 'static>(
        &self,
        query: &str,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>> {
        let query = query.to_string();
        Ok(self
            .interact(move |conn| {
                conn.prepare(query.as_str())
                    .unwrap()
                    .query_map([], T::from_row)
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
            })
            .await??)
    }

    async fn query_single<T: FromRow + Send + Sync + 'static>(
        &self,
        query: &str,
    ) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let query = query.to_string();
        Ok(self
            .interact(move |conn| {
                conn.prepare(query.as_str())
                    .unwrap()
                    .query_row([], T::from_row)
                    .optional()
            })
            .await??)
    }
}

impl UserDatabase for DbConn {
    async fn get_user(
        &self,
        username: &str,
    ) -> Result<Option<UserEntity>, Box<dyn std::error::Error>> {
        let username = NormalizedString::new(username);
        Ok(self
            .query_single::<UserEntity>(&format!(
                "SELECT * FROM users WHERE normalized_name = '{}' LIMIT 1",
                *username
            ))
            .await?)
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
        Ok(self.query::<UserEntity>("SELECT * FROM users").await?)
    }
}

impl SensorDatabase for DbConn {
    async fn get_sensors(&self) -> Result<Vec<Sensor>, Box<dyn std::error::Error>> {
        Ok(self
            .query::<SensorEntity>("SELECT * FROM sensors")
            .await?
            .into_iter()
            .map(SensorEntity::into)
            .collect())
    }
}
