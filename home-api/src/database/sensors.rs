use super::{Database, DbConn};
use crate::models::db::SensorEntity;
use home_common::models::Sensor;

pub trait SensorDatabase {
    async fn get_sensors(&self) -> Result<Vec<Sensor>, Box<dyn std::error::Error>>;
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
