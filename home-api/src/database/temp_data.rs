use super::{Database, DbConn};
use crate::models::db::TempDataEntry;

pub trait TempDataDatabase {
    async fn get_temp_data(
        &self,
        host: Option<Vec<impl Into<String>>>,
        limit: Option<usize>,
        offset: Option<usize>,
        after: Option<i64>,
    ) -> Result<Vec<TempDataEntry>, anyhow::Error>;
    async fn create_temp_data_batch(
        &self,
        entries: Vec<TempDataEntry>,
    ) -> Result<usize, anyhow::Error>;
}

impl TempDataDatabase for DbConn {
    async fn get_temp_data(
        &self,
        host: Option<Vec<impl Into<String>>>,
        limit: Option<usize>,
        offset: Option<usize>,
        after: Option<i64>,
    ) -> Result<Vec<TempDataEntry>, anyhow::Error> {
        let mut query = String::from("SELECT * FROM sensor_temp_data");
        if let Some(host) = host {
            query.push_str(" WHERE host IN ('");
            query.push_str(
                host.into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .join("', '")
                    .as_str(),
            );
            query.push_str("')");
            if let Some(after) = after {
                query.push_str(" AND timestamp > ");
                query.push_str(&after.to_string());
            }
        } else if let Some(after) = after {
            query.push_str(" WHERE timestamp > ");
            query.push_str(&after.to_string());
        }
        query.push_str(" ORDER BY timestamp DESC");
        if let Some(limit) = limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }
        if let Some(offset) = offset {
            query.push_str(" OFFSET ");
            query.push_str(&offset.to_string());
        }

        self.query::<TempDataEntry>(&query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn create_temp_data_batch(
        &self,
        entries: Vec<TempDataEntry>,
    ) -> Result<usize, anyhow::Error> {
        if entries.is_empty() {
            return Ok(0);
        }
        let mut query = String::from(
            "INSERT INTO sensor_temp_data(host, timestamp, temperature, humidity) \nVALUES ",
        );
        for entry in entries {
            query.push_str(&format!(
                "('{}', {}, {}, {}),\n",
                entry.host, entry.timestamp, entry.temperature, entry.humidity,
            ));
        }
        query.pop();
        query.pop();
        query.push_str("\nON CONFLICT(host, timestamp) DO UPDATE SET temperature = excluded.temperature, humidity = excluded.humidity;");
        self.execute(&query)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}
