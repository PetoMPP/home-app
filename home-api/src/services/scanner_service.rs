use super::sensor_service::SensorService;
use crate::models::db::SensorEntity;
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
}

#[derive(Default)]
pub struct ScannerService {
    pub last_result: Option<ScannerResult>,
    handle: Option<JoinHandle<Result<ScannerResult, &'static str>>>,
    progress: Arc<Mutex<ScanProgress>>,
}

impl ScannerService {
    async fn scan_inner(progress: Arc<Mutex<ScanProgress>>) -> Result<ScannerResult, &'static str> {
        let Some(target) = pnet::datalink::interfaces().into_iter().find_map(|n| {
            n.ips
                .into_iter()
                .map(|ip| ip.ip())
                .find(|ip| ip.is_ipv4() && !ip.is_loopback() && !ip.is_unspecified())
                .map(|ip| ip.to_string())
        }) else {
            return Err("No network to scan!");
        };
        let target = &target[..target.rfind('.').unwrap() + 1];
        {
            let mut scan_progress = progress.lock().await;
            scan_progress.target = target.to_string();
            scan_progress.total = 256;
        }
        let mut sensors = vec![];
        for i in 0..=255 {
            progress.lock().await.progress = i + 1;
            let host = format!("{}{}", target, i);
            let Ok(sensor) = reqwest::Client::new().get_sensor(&host).await else {
                continue;
            };
            let sensor_entity = SensorEntity {
                name: sensor.name.to_string(),
                location: sensor.location.to_string(),
                features: sensor.features,
                host,
                pair_id: None,
            };
            sensors.push(sensor_entity);
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
