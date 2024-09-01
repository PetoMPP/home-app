use super::{sensors::SensorDatabase, Database, DbConn};
use crate::models::{db::AreaEntity, Area};

pub trait AreaDatabase {
    async fn get_area_entities(&self) -> Result<Vec<AreaEntity>, anyhow::Error>;
    async fn get_areas(&self) -> Result<Vec<Area>, anyhow::Error>;
    async fn get_area(&self, id: i64) -> Result<Area, anyhow::Error>;
    async fn create_area(&self, area: AreaEntity) -> Result<AreaEntity, anyhow::Error>;
    async fn update_area(&self, area: AreaEntity) -> Result<AreaEntity, anyhow::Error>;
    async fn delete_area(&self, id: i64) -> Result<bool, anyhow::Error>;
}

impl AreaDatabase for DbConn {
    async fn get_area_entities(&self) -> Result<Vec<AreaEntity>, anyhow::Error> {
        Ok(self
            .query::<AreaEntity>("SELECT rowid, name FROM areas")
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?)
    }

    async fn get_areas(&self) -> Result<Vec<Area>, anyhow::Error> {
        let area_entities = self.get_area_entities().await?;
        let mut areas = vec![];
        for area in area_entities {
            let sensors = self.get_sensors_by_area_id(area.id).await?;
            areas.push(Area {
                id: area.id,
                name: area.name,
                sensors,
            });
        }
        Ok(areas)
    }

    async fn get_area(&self, id: i64) -> Result<Area, anyhow::Error> {
        let area_entity = self
            .query_single::<AreaEntity>(&format!(
                "SELECT rowid, name FROM areas WHERE rowid = {}",
                id
            ))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or(anyhow::anyhow!("No area found"))?;
        let sensors = self.get_sensors_by_area_id(area_entity.id).await?;
        Ok(Area {
            id: area_entity.id,
            name: area_entity.name,
            sensors,
        })
    }

    async fn create_area(&self, area: AreaEntity) -> Result<AreaEntity, anyhow::Error> {
        Ok(self
            .query_single::<AreaEntity>(&format!(
                "INSERT INTO areas (name) VALUES ('{}') RETURNING rowid, name",
                area.name
            ))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or(anyhow::anyhow!("No area created"))?)
    }

    async fn update_area(&self, area: AreaEntity) -> Result<AreaEntity, anyhow::Error> {
        Ok(self
            .query_single::<AreaEntity>(&format!(
                "UPDATE areas SET name = '{}' WHERE rowid = {} RETURNING rowid, name",
                area.name, area.id
            ))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .ok_or(anyhow::anyhow!("No area updated"))?)
    }

    async fn delete_area(&self, id: i64) -> Result<bool, anyhow::Error> {
        Ok(self
            .execute(&format!("DELETE FROM areas WHERE rowid = {}", id))
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            > 0)
    }
}
