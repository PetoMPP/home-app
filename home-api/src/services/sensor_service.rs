use super::http_client::HttpRequest;
use crate::models::{
    db::{SensorEntity, SensorFeatures},
    json::{
        ErrorResponse, Measurement, MeasurementsResponse, PairResponse, SensorDto, SensorFormData,
        SensorResponse,
    },
};
use anyhow::anyhow;
use serde::Serialize;
use std::{error::Error, time::Duration};

const SENSOR_PORT: u16 = 42069;
const PAIR_HEADER_NAME: &str = "X-Pair-Id";

pub trait SensorService {
    async fn get_sensor(
        &self,
        host: &str,
    ) -> Result<Result<SensorEntity, String>, Box<dyn Error + Send + Sync>>;
    async fn pair(&self, host: &str) -> Result<SensorEntity, Box<dyn Error + Send + Sync>>;
    async fn update_sensor(
        &self,
        host: &str,
        pair_id: &str,
        sensor: SensorFormData,
    ) -> Result<SensorResponse, Box<dyn Error + Send + Sync>>;
}

impl SensorService for reqwest::Client {
    async fn get_sensor(
        &self,
        host: &str,
    ) -> Result<Result<SensorEntity, String>, Box<dyn Error + Send + Sync>> {
        let host_uri = format!("http://{}:{}/", host, SENSOR_PORT);
        let response = self
            .get(host_uri.clone() + "sensor")
            .timeout(std::time::Duration::from_secs_f32(0.2))
            .send_parse_retry::<SensorResponse, ErrorResponse>(3)
            .await?
            .map_err(|e| anyhow!("{}", e.error))?;

        let sensor_entity = SensorEntity {
            name: response.name.to_string(),
            location: response.location.to_string(),
            features: SensorFeatures::from_bits_retain(response.features),
            host: host.to_string(),
            pair_id: None,
        };

        Ok(Ok(sensor_entity))
    }

    async fn pair(&self, host: &str) -> Result<SensorEntity, Box<dyn Error + Send + Sync>> {
        let host_uri = format!("http://{}:{}/", host, SENSOR_PORT);
        let response = self
            .post(host_uri.clone() + "pair")
            .send_parse_retry::<PairResponse, ErrorResponse>(3)
            .await?
            .map_err(|e| anyhow!("{}", e.error))?;

        let id = response.id;

        // Wait for the sensor to reopen the socket
        tokio::time::sleep(Duration::from_secs_f32(0.2)).await;

        let response = self
            .post(host_uri.clone() + "pair/confirm")
            .header(PAIR_HEADER_NAME, id.as_str())
            .send_parse_err_retry::<ErrorResponse>(3)
            .await?
            .map_err(|e| e.error.to_string())?;

        if response.is_success() {
            // Wait for the sensor to reopen the socket
            tokio::time::sleep(Duration::from_secs_f32(0.2)).await;
            let mut sensor = self.get_sensor(host).await??;
            sensor.pair_id = Some(id.to_string());
            Ok(sensor)
        } else {
            Err("Pairing failed".into())
        }
    }

    async fn update_sensor(
        &self,
        host: &str,
        pair_id: &str,
        sensor: SensorFormData,
    ) -> Result<SensorResponse, Box<dyn Error + Send + Sync>> {
        let host_uri = format!("http://{}:{}/", host, SENSOR_PORT);
        let sensor_dto = sensor.into();
        let response = self
            .post(host_uri.clone() + "sensor")
            .header(PAIR_HEADER_NAME, pair_id)
            .json::<SensorDto>(&sensor_dto)
            .send_parse_err_retry::<ErrorResponse>(3)
            .await?
            .map_err(|e| e.error.to_string());

        let response = response?;

        if response.is_success() {
            // Wait for the sensor to reopen the socket
            tokio::time::sleep(Duration::from_secs_f32(0.2)).await;
            let response = self
                .get(host_uri.clone() + "sensor")
                .send_parse_retry::<SensorResponse, ErrorResponse>(3)
                .await?
                .map_err(|e| e.error.to_string())?;

            return Ok(response);
        }

        Err("Update failed".into())
    }
}

pub trait TempSensorService {
    async fn get_temp(
        &self,
        host: &str,
        pair_id: &str,
        count: Option<u64>,
        max_age: Option<u64>,
    ) -> Result<Vec<Measurement>, anyhow::Error>;
}

#[derive(Serialize)]
struct TempRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<u64>,
}

impl TempSensorService for reqwest::Client {
    async fn get_temp(
        &self,
        host: &str,
        pair_id: &str,
        count: Option<u64>,
        max_age: Option<u64>,
    ) -> Result<Vec<Measurement>, anyhow::Error> {
        let host_uri = format!("http://{}:{}/", host, SENSOR_PORT);
        let response = self
            .get(host_uri.clone() + "dht")
            .header(PAIR_HEADER_NAME, pair_id)
            .json(&TempRequest {
                count,
                timestamp: max_age,
            })
            .send_parse_retry::<MeasurementsResponse, ErrorResponse>(3)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .map_err(|e| anyhow::anyhow!("{}", e.error))?;

        Ok(response.measurements)
    }
}
