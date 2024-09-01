use r2d2_sqlite::rusqlite::OptionalExtension;

pub mod areas;
pub mod data_schedule;
pub mod sensors;
pub mod temp_data;
pub mod user_sessions;
pub mod users;

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
