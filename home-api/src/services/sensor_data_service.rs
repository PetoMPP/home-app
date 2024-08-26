use crate::{
    database::{
        data_schedule::DataScheduleDatabase, sensors::SensorDatabase, temp_data::TempDataDatabase,
        DbPool,
    },
    models::db::{DataScheduleEntry, SensorFeatures, TempDataEntry},
};
use chrono::Utc;
use std::{sync::Arc, time::SystemTime};
use tokio::{
    task::JoinHandle,
    time::{self, Instant},
};

use super::sensor_service::TempSensorService;

pub struct SensorDataService {
    handle: Option<JoinHandle<()>>,
    runtime: Arc<tokio::runtime::Runtime>,
    current_schedule: Option<Vec<DataScheduleEntry>>,
    pool: DbPool,
}

impl SensorDataService {
    pub fn new(runtime: Arc<tokio::runtime::Runtime>, pool: DbPool) -> Self {
        Self {
            handle: Default::default(),
            runtime,
            current_schedule: Default::default(),
            pool,
        }
    }

    pub async fn init(&mut self) -> Result<(), anyhow::Error> {
        let schedule = self.pool.get().await?.get_schedule().await?;
        self.current_schedule = Some(schedule);
        self.start();
        Ok(())
    }

    fn start(&mut self) {
        let Some(schedule) = self.current_schedule.as_ref() else {
            return;
        };
        let schedule = schedule.clone();
        let pool = self.pool.clone();
        let handle = self.runtime.spawn(async move {
            let schedule = schedule.clone();
            let mut handles = tokio::task::JoinSet::<Result<(), anyhow::Error>>::new();
            for entry in schedule.into_iter() {
                let pool = pool.clone();
                handles.spawn(async move {
                    let pool = pool.clone();
                    let mut last_dur = time::Duration::from_millis(
                        SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64
                            % entry.interval_ms,
                    );
                    loop {
                        time::sleep(time::Duration::from_millis(entry.interval_ms) - last_dur)
                            .await;
                        let start = Instant::now();
                        Self::collect_data(&entry, &pool).await?;

                        last_dur = start.elapsed();
                    }
                });
            }

            while let Some(e) = handles.join_next().await {
                // nothing should finish
                // restart service
                eprintln!("Sensor data service task finished unexpectedly: {:?}", e);
            }
        });

        self.handle = Some(handle);
    }

    async fn collect_data(entry: &DataScheduleEntry, pool: &DbPool) -> Result<(), anyhow::Error> {
        if entry.features.contains(SensorFeatures::TEMPERATURE) {
            let sensors = pool
                .get()
                .await?
                .get_sensors_by_features(SensorFeatures::TEMPERATURE)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            for sensor in sensors {
                const SAVE_INTERVAL: i64 = 1000 * 60 * 15; // 15 minutes
                const MAX_COUNT: i64 = 150;
                let last_measurement = pool
                    .get()
                    .await?
                    .get_temp_data(Some(&sensor.host), Some(1), None)
                    .await?
                    .first()
                    .map_or(0, |t| t.timestamp);
                let count = (Utc::now().timestamp() - last_measurement as i64) / SAVE_INTERVAL + 4;
                let count = count.min(MAX_COUNT);
                let client = reqwest::Client::new();
                let measurements = client
                    .get_temp(&sensor.host, Some(count as u64), None)
                    .await?;
                let _ = pool
                    .get()
                    .await?
                    .create_temp_data_batch(
                        measurements
                            .into_iter()
                            .map(|m| TempDataEntry {
                                host: sensor.host.clone(),
                                timestamp: m.timestamp,
                                temperature: m.temperature,
                                humidity: m.humidity,
                            })
                            .collect(),
                    )
                    .await?;
            }
        }
        if entry.features.contains(SensorFeatures::MOTION) {
            // do motion
        }

        Ok(())
    }
}