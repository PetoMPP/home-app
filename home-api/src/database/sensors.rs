use super::{Database, DbConn};
use crate::models::db::SensorEntity;

pub trait SensorDatabase {
    async fn get_sensor(
        &self,
        host: &str,
    ) -> Result<Option<SensorEntity>, Box<dyn std::error::Error>>;
    async fn get_sensors(&self) -> Result<Vec<SensorEntity>, Box<dyn std::error::Error>>;
    async fn create_sensor(
        &self,
        sensor: SensorEntity,
    ) -> Result<SensorEntity, Box<dyn std::error::Error>>;
    async fn delete_sensor(&self, host: &str) -> Result<(), Box<dyn std::error::Error>>;
}

impl SensorDatabase for DbConn {
    async fn get_sensor(&self, host: &str) -> Result<Option<SensorEntity>, Box<dyn std::error::Error>> {
        Ok(self
            .query_single(&format!("SELECT * FROM sensors WHERE host = '{}'", host))
            .await?)
    }

    async fn get_sensors(&self) -> Result<Vec<SensorEntity>, Box<dyn std::error::Error>> {
        Ok(self.query::<SensorEntity>("SELECT * FROM sensors").await?)
    }

    async fn create_sensor(
        &self,
        sensor: SensorEntity,
    ) -> Result<SensorEntity, Box<dyn std::error::Error>> {
        Ok(self.query_single(
            &format!("INSERT INTO sensors (name, location, features, host, pair_id) VALUES ('{}', '{}', {}, '{}', '{}') RETURNING *",
            sensor.name,
            sensor.location,
            sensor.features,
            sensor.host,
            sensor.pair_id.ok_or("pair_id must be set")?),
        ).await?.ok_or("Error creating sensor")?)
    }

    async fn delete_sensor(&self, host: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.execute(&format!("DELETE FROM sensors WHERE host = '{}'", host))
            .await?;
        Ok(())
    }
}
