use super::{Database, DbConn};
use crate::models::db::{SensorEntity, SensorFeatures};

pub trait SensorDatabase {
    async fn get_sensor(
        &self,
        host: &str,
    ) -> Result<Option<SensorEntity>, Box<dyn std::error::Error>>;
    async fn get_sensors(&self) -> Result<Vec<SensorEntity>, Box<dyn std::error::Error>>;
    async fn get_sensors_by_features(
        &self,
        features: SensorFeatures,
    ) -> Result<Vec<SensorEntity>, Box<dyn std::error::Error>>;
    async fn get_sensors_by_area_id(
        &self,
        area_id: i64,
    ) -> Result<Vec<SensorEntity>, anyhow::Error>;
    async fn create_sensor(
        &self,
        sensor: SensorEntity,
    ) -> Result<SensorEntity, Box<dyn std::error::Error>>;
    async fn update_sensor(
        &self,
        host: &str,
        sensor: SensorEntity,
    ) -> Result<bool, Box<dyn std::error::Error>>;
    async fn delete_sensor(&self, host: &str) -> Result<usize, Box<dyn std::error::Error>>;
}

impl SensorDatabase for DbConn {
    async fn get_sensor(
        &self,
        host: &str,
    ) -> Result<Option<SensorEntity>, Box<dyn std::error::Error>> {
        self.query_single(&format!("SELECT * FROM sensors LEFT JOIN areas ON sensors.area_id = areas.rowid WHERE sensors.host = '{}'", host))
            .await
    }

    async fn get_sensors(&self) -> Result<Vec<SensorEntity>, Box<dyn std::error::Error>> {
        self.query::<SensorEntity>(
            "SELECT * FROM sensors LEFT JOIN areas ON sensors.area_id = areas.rowid",
        )
        .await
    }

    async fn get_sensors_by_features(
        &self,
        features: SensorFeatures,
    ) -> Result<Vec<SensorEntity>, Box<dyn std::error::Error>> {
        let features = features.bits();
        self.query::<SensorEntity>(&format!(
            "SELECT * FROM sensors LEFT JOIN areas ON sensors.area_id = areas.rowid WHERE sensors.features & {features} = {features}"
        ))
        .await
    }

    async fn get_sensors_by_area_id(
        &self,
        area_id: i64,
    ) -> Result<Vec<SensorEntity>, anyhow::Error> {
        self.query::<SensorEntity>(&format!(
            "SELECT * FROM sensors LEFT JOIN areas ON sensors.area_id = areas.rowid WHERE sensors.area_id = {}",
            area_id
        ))
        .await.map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn create_sensor(
        &self,
        sensor: SensorEntity,
    ) -> Result<SensorEntity, Box<dyn std::error::Error>> {
        Ok(self.query_single(
            &format!("INSERT INTO sensors (name, area_id, features, host, pair_id) VALUES ('{}', {}, {}, '{}', '{}') RETURNING *",
            sensor.name,
            sensor.area.map(|a| a.id.to_string()).unwrap_or("NULL".to_string()),
            sensor.features.bits(),
            sensor.host,
            sensor.pair_id.ok_or("pair_id must be set")?),
        ).await?.ok_or("Error creating sensor")?)
    }

    async fn delete_sensor(&self, host: &str) -> Result<usize, Box<dyn std::error::Error>> {
        self.execute(&format!("DELETE FROM sensors WHERE host = '{}'", host))
            .await
    }

    async fn update_sensor(
        &self,
        host: &str,
        sensor: SensorEntity,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self
            .execute(&format!(
                "UPDATE sensors SET name = '{}', area_id = {}, features = {} WHERE host = '{}'",
                sensor.name,
                sensor
                    .area
                    .map(|a| a.id.to_string())
                    .unwrap_or("NULL".to_string()),
                sensor.features.bits(),
                host
            ))
            .await?
            > 0)
    }
}
