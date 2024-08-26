use super::{Database, DbConn};
use crate::models::db::{DataSchedule, DataScheduleEntry};

pub trait DataScheduleDatabase {
    async fn create_entry(
        &self,
        entry: DataScheduleEntry,
    ) -> Result<DataScheduleEntry, anyhow::Error>;
    async fn get_schedule(&self) -> Result<DataSchedule, anyhow::Error>;
    async fn update_entry(
        &self,
        src: DataScheduleEntry,
        entry: DataScheduleEntry,
    ) -> Result<DataScheduleEntry, anyhow::Error>;
    async fn delete_entry(&self, entry: DataScheduleEntry) -> Result<bool, anyhow::Error>;
}

impl DataScheduleDatabase for DbConn {
    async fn create_entry(
        &self,
        entry: DataScheduleEntry,
    ) -> Result<DataScheduleEntry, anyhow::Error> {
        Ok(self
            .query_single::<DataScheduleEntry>(&format!(
                "INSERT INTO data_schedule (features, interval_ms) VALUES ('{}', '{}') RETURNING *",
                entry.features.bits(),
                entry.interval_ms
            ))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or(anyhow::anyhow!("Failed to create entry"))?)
    }

    async fn get_schedule(&self) -> Result<DataSchedule, anyhow::Error> {
        Ok(self
            .query::<DataScheduleEntry>("SELECT * FROM data_schedule")
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?)
    }

    async fn update_entry(
        &self,
        src: DataScheduleEntry,
        entry: DataScheduleEntry,
    ) -> Result<DataScheduleEntry, anyhow::Error> {
        Ok(self
            .query_single::<DataScheduleEntry>(&format!(
                "UPDATE data_schedule SET features = '{}', interval_ms = '{}' WHERE features = '{}' AND interval_ms = '{}' RETURNING *",
                entry.features.bits(),
                entry.interval_ms,
                src.features.bits(),
                src.interval_ms
            ))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or(anyhow::anyhow!("Failed to update entry"))?)
    }

    async fn delete_entry(&self, entry: DataScheduleEntry) -> Result<bool, anyhow::Error> {
        Ok(self
            .execute(&format!(
                "DELETE FROM data_schedule WHERE features = '{}' AND interval_ms = '{}'",
                entry.features.bits(),
                entry.interval_ms
            ))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            > 0)
    }
}
