use crate::database::DbPool;
use serde_derive::{Deserialize, Serialize};
use std::{future::Future, sync::Arc};
use tokio::{sync::Mutex, task::JoinHandle};

pub trait Scannable: Send + Sync + Clone + Default + std::fmt::Debug + 'static {
    type Error: Send + Sync;
    fn scan(
        client: &reqwest::Client,
        host: &str,
    ) -> impl Future<
        Output = Result<Result<Self, Self::Error>, Box<dyn std::error::Error + Send + Sync>>,
    > + Send;
    fn check(
        &mut self,
        pool: &DbPool,
    ) -> impl Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScanProgress<T: Scannable> {
    pub progress: u32,
    pub scanned: Vec<T>,
    pub total: u32,
    pub target: String,
}

impl<T: Scannable> ScanProgress<T> {
    pub fn text(&self) -> String {
        format!("Scanned {} of {} hosts", self.progress, self.total)
    }
}

#[derive(Clone, Debug)]
pub enum ScannerState<T: Scannable> {
    Idle(Option<ScannerResult<T>>),
    Scanning(ScanProgress<T>),
    Error(String),
}

impl<T: Scannable> ScannerState<T> {
    pub fn scanned(&self) -> Vec<T> {
        match self {
            ScannerState::Idle(Some(result)) => result.scanned.clone(),
            ScannerState::Scanning(progress) => progress.scanned.clone(),
            _ => vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScannerResult<T> {
    pub scanned: Vec<T>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub duration: chrono::Duration,
}

impl<T: Scannable> ScannerResult<T> {
    pub fn created_display(&self) -> String {
        self.created.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    pub fn duration_display(&self) -> String {
        format!(
            "{:02}:{:02}.{:03}",
            self.duration.num_minutes(),
            self.duration.num_seconds() % 60,
            self.duration.num_milliseconds() % 1000
        )
    }
}

pub struct ScannerService<T: Scannable> {
    pub last_result: Option<ScannerResult<T>>,
    handle: Option<JoinHandle<Result<ScannerResult<T>, String>>>,
    progress: Arc<Mutex<ScanProgress<T>>>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl<T: Scannable> ScannerService<T> {
    pub fn new(runtime: Arc<tokio::runtime::Runtime>) -> Self {
        Self {
            last_result: Default::default(),
            handle: Default::default(),
            progress: Default::default(),
            runtime,
        }
    }

    async fn scan_inner(
        progress: Arc<Mutex<ScanProgress<T>>>,
        pool: DbPool,
    ) -> Result<ScannerResult<T>, String> {
        let started = chrono::Utc::now();
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
            let pool = pool.clone();
            let task = tokio::spawn(async move {
                let host = format!("{}{}", target, i);
                let client = reqwest::Client::new();
                if let Ok(Ok(mut scanned)) = T::scan(&client, &host).await {
                    scanned.check(&pool).await.ok();
                    progress.lock().await.scanned.push(scanned);
                }
                progress.lock().await.progress += 1;
            });
            handles.spawn(task);
        }

        while handles.join_next().await.is_some() {}

        let created = chrono::Utc::now();
        let duration = created - started;

        Ok(ScannerResult {
            scanned: progress.lock().await.scanned.clone(),
            created,
            duration,
        })
    }

    pub async fn init(&mut self, pool: DbPool) -> ScannerState<T> {
        self.progress = Default::default();
        let progress = self.progress.clone();
        if self.handle.is_none() {
            self.handle = Some(self.runtime.spawn(Self::scan_inner(progress.clone(), pool)));
        }

        self.state().await
    }

    pub async fn cancel(&mut self) {
        if let Some(handle) = &mut self.handle {
            handle.abort();
            self.handle = None;
        }
    }

    pub async fn state(&mut self) -> ScannerState<T> {
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
