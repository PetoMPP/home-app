use crate::Sensor;
use deref_derive::{Deref, DerefMut};
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone, Deref, DerefMut)]
pub struct SqlitePool(pub r2d2::Pool<SqliteConnectionManager>);

impl SqlitePool {
    pub fn get_sensors(&self) -> Result<Vec<Sensor>, r2d2_sqlite::rusqlite::Error> {
        let conn = self.get().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM sensors").unwrap();
        let sensors = stmt
            .query_map([], |row| {
                Ok(Sensor {
                    id: row.get(0)?,
                    features: row.get(1)?,
                })
            })?
            .filter_map(|s| s.ok())
            .collect::<Vec<_>>();
        Ok(sensors)
    }
}
