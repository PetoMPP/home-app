use std::str::FromStr;

use deref_derive::{Deref, DerefMut};
use home_common::models::Sensor;
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
                    name: heapless::String::<64>::from_str(row.get::<_, String>(0)?.as_str())
                        .unwrap(),
                    location: heapless::String::<64>::from_str(row.get::<_, String>(1)?.as_str())
                        .unwrap(),
                    features: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>();

        sensors
    }
}
