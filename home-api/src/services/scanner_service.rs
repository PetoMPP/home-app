use crate::models::Sensor;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Clone, Debug)]
pub enum ScannerState {
    Idle(Option<ScannerResult>),
    Scanning(f32),
    Done(ScannerResult),
    Error(()),
}

#[derive(Clone, Debug)]
pub struct ScannerResult {
    pub sensors: Vec<Sensor>,
    pub created: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct ScannerService {
    pub last_result: Option<ScannerResult>,
    handle: Option<JoinHandle<Result<ScannerResult, &'static str>>>,
    progress: Arc<Mutex<f32>>,
}

impl ScannerService {
    async fn scan_inner(progress: Arc<Mutex<f32>>) -> Result<ScannerResult, &'static str> {
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
        println!("Scanning network: {}", target);
        let mut sensors = vec![];
        for i in 0..=255 {
            *progress.lock().await = i as f32 / 255.0;
            let host = format!("{}{}", target, i);
            const PORT: u16 = 42069;
            let Ok(resp) = reqwest::Client::new()
                .get(format!("http://{}:{}", host, PORT))
                .timeout(Duration::from_secs(1))
                .send()
                .await
            else {
                continue;
            };
            let Ok(sensor) = resp.json::<Sensor>().await else {
                continue;
            };
            sensors.push(sensor);
        }
        Ok(ScannerResult {
            sensors,
            created: chrono::Utc::now(),
        })
    }

    pub async fn init(&mut self) -> ScannerState {
        let progress = self.progress.clone();
        if self.handle.is_none() {
            self.handle = Some(tokio::spawn(async move {
                Self::scan_inner(progress.clone()).await
            }));
        }

        self.state().await
    }

    pub async fn state(&mut self) -> ScannerState {
        let Some(handle) = &mut self.handle else {
            return ScannerState::Idle(self.last_result.clone());
        };

        if handle.is_finished() {
            let state = match handle.await {
                Ok(sensors) => match sensors {
                    Ok(sensors) => ScannerState::Done(sensors),
                    Err(_) => ScannerState::Error(()),
                },
                Err(_) => ScannerState::Error(()),
            };
            self.handle = None;
            return state;
        }

        ScannerState::Scanning(*self.progress.lock().await)
    }
}
