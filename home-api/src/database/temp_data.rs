use super::{Database, DbConn};
use crate::models::db::TempDataEntry;
use sqlx::Execute;

pub trait TempDataDatabase {
    async fn get_temp_data(&self, host: Option<impl Into<String>>, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<TempDataEntry>, anyhow::Error>;
    async fn create_temp_data(&self, entry: TempDataEntry) -> Result<TempDataEntry, anyhow::Error>;
    async fn create_temp_data_batch(
        &self,
        entries: Vec<TempDataEntry>,
    ) -> Result<usize, anyhow::Error>;
}

impl TempDataDatabase for DbConn {
    async fn get_temp_data(&self, host: Option<impl Into<String>>, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<TempDataEntry>, anyhow::Error> {
        let mut builder = sqlx::QueryBuilder::new("SELECT * FROM temp_data");
        if let Some(host) = host {
            builder.push("WHERE host = ");
            builder.push_bind(host.into());
        }
        if let Some(limit) = limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }
        if let Some(offset) = offset {
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }
        builder.push(" ORDER BY timestamp DESC");

        Ok(self
            .query::<TempDataEntry>(builder.build().sql())
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?)
    }

    async fn create_temp_data(&self, entry: TempDataEntry) -> Result<TempDataEntry, anyhow::Error> {
        Ok(self
            .query_single::<TempDataEntry>(&format!(
                "INSERT INTO temp_data (host, timestamp, temperature, humidity) VALUES ('{}', {}, {}, {}) RETURNING *",
                entry.host,
                entry.timestamp,
                entry.temperature,
                entry.humidity,
            ))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or(anyhow::anyhow!("Failed to create temp data"))?)
    }

    async fn create_temp_data_batch(
        &self,
        entries: Vec<TempDataEntry>,
    ) -> Result<usize, anyhow::Error> {
        let mut builder = sqlx::QueryBuilder::new(
            "INSERT INTO temp_data(host, timestamp, temperature, humidity) ",
        );
        builder.push_values(entries, |mut b, temp| {
            b.push_bind(temp.host)
                .push_bind(temp.timestamp as u32)
                .push_bind(temp.temperature)
                .push_bind(temp.humidity);
        });
        builder.build().sql();
        Ok(self
            .execute(&builder.build().sql())
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?)
    }
}
