use super::sensor_service::SensorService;
use crate::{
    database::{sensors::SensorDatabase, DbPool},
    models::db::SensorEntity,
};
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanProgress {
    pub progress: u32,
    pub total: u32,
    pub target: String,
}

impl ScanProgress {
    pub fn text(&self) -> String {
        format!("Scanned {} of {} hosts", self.progress, self.total)
    }

    pub fn style(&self) -> String {
        format!(
            "--value:{};--size:16rem;--thickness:0.5rem;",
            (self.progress as f32 / self.total as f32 * 100.0) as u32
        )
    }
}

#[derive(Clone, Debug)]
pub enum ScannerState {
    Idle(Option<ScannerResult>),
    Scanning(ScanProgress),
    Error(String),
}

#[derive(Clone, Debug)]
pub struct ScannerResult {
    pub sensors: Vec<SensorEntity>,
    pub created: chrono::DateTime<chrono::Utc>,
}

impl ScannerResult {
    pub fn created_display(&self) -> String {
        self.created.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    pub async fn check_sensors(&mut self, pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
        for s in self.sensors.iter_mut() {
            s.pair_id = pool
                .get()
                .await
                .map_err(|e| e.to_string())?
                .get_sensor(&s.host)
                .await
                .map_err(|e| e.to_string())?
                .and_then(|s| s.pair_id);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct ScannerService {
    pub last_result: Option<ScannerResult>,
    handle: Option<JoinHandle<Result<ScannerResult, String>>>,
    progress: Arc<Mutex<ScanProgress>>,
}

impl ScannerService {
    async fn scan_inner(progress: Arc<Mutex<ScanProgress>>) -> Result<ScannerResult, String> {
        let Some(target) = pnet::datalink::interfaces().into_iter().find_map(|n| {
            n.ips
                .into_iter()
                .map(|ip| ip.ip())
                .find(|ip| ip.is_ipv4() && !ip.is_loopback() && !ip.is_unspecified())
                .map(|ip| ip.to_string())
        }) else {
            return Err("No network to scan!".to_string());
        };
        let target = target[..target.rfind('.').unwrap() + 1].to_string();
        {
            let mut scan_progress = progress.lock().await;
            scan_progress.target = target.to_string();
            scan_progress.total = 256;
        }
        let mut handles = tokio::task::JoinSet::new();
        for i in 0..=255 {
            let progress = progress.clone();
            let target = target.clone();
            let task = tokio::spawn(async move {
                progress.lock().await.progress = i + 1;
                let host = format!("{}{}", target, i);
                reqwest::Client::new().get_sensor(&host).await.ok()
            });
            handles.spawn(task);
        }

        let mut sensors = vec![];
        while let Some(sensor) = handles.join_next().await {
            if let Some(sensor) = sensor.and_then(|r| r).map_err(|e| e.to_string())? {
                sensors.push(sensor);
            }
        }

        Ok(ScannerResult {
            sensors,
            created: chrono::Utc::now(),
        })
    }

    pub async fn init(&mut self) -> ScannerState {
        self.progress = Default::default();
        let progress = self.progress.clone();
        if self.handle.is_none() {
            self.handle = Some(tokio::spawn(async move {
                Self::scan_inner(progress.clone()).await
            }));
        }

        self.state().await
    }

    pub async fn cancel(&mut self) {
        if let Some(handle) = &mut self.handle {
            handle.abort();
            self.handle = None;
        }
    }

    pub async fn state(&mut self) -> ScannerState {
        let Some(handle) = &mut self.handle else {
            return ScannerState::Idle(self.last_result.clone());
        };

        if handle.is_finished() {
            let state = match handle.await {
                Ok(sensors) => match sensors {
                    Ok(sensors) => {
                        self.last_result = Some(sensors);
                        ScannerState::Idle(self.last_result.clone())
                    }
                    Err(e) => ScannerState::Error(e.to_string()),
                },
                Err(e) => ScannerState::Error(e.to_string()),
            };
            self.handle = None;
            return state;
        }

        ScannerState::Scanning(self.progress.lock().await.clone())
    }
}
