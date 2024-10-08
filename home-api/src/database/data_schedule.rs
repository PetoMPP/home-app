use super::{Database, DbConn};
use crate::models::db::{DataSchedule, DataScheduleEntry};

pub trait DataScheduleDatabase {
    async fn create_entry(
        &self,
        entry: DataScheduleEntry,
    ) -> Result<Option<DataScheduleEntry>, anyhow::Error>;
    async fn get_schedule(&self) -> Result<DataSchedule, anyhow::Error>;
    async fn delete_entry(&self, entry: DataScheduleEntry) -> Result<bool, anyhow::Error>;
}

impl DataScheduleDatabase for DbConn {
    async fn create_entry(
        &self,
        entry: DataScheduleEntry,
    ) -> Result<Option<DataScheduleEntry>, anyhow::Error> {
        self.query_single::<DataScheduleEntry>(&format!(
            "INSERT INTO data_schedule (features, interval_ms)\n\
                VALUES ('{}', '{}')\n\
                ON CONFLICT(features)\n\
                DO\n\
                  UPDATE SET interval_ms = excluded.interval_ms\n\
                  WHERE interval_ms != excluded.interval_ms\n\
                  RETURNING *",
            entry.features.bits(),
            entry.interval_ms
        ))
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn get_schedule(&self) -> Result<DataSchedule, anyhow::Error> {
        self.query::<DataScheduleEntry>("SELECT * FROM data_schedule")
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
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
