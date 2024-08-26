use super::http_client::HttpRequest;
use crate::models::{
    db::{SensorEntity, SensorFeatures},
    json::{
        ErrorResponse, Measurement, MeasurementsResponse, PairResponse, SensorDto, SensorFormData,
        SensorResponse,
    },
};
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
        let Ok(response) = self
            .get(host_uri.clone() + "sensor")
            .timeout(std::time::Duration::from_secs_f32(0.2))
            .send_parse::<SensorResponse, ErrorResponse>()
            .await?
        else {
            return Err("No sensor found".into());
        };

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
            .send_parse::<PairResponse, ErrorResponse>()
            .await?
            .map_err(|e| e.error.to_string())?;

        let id = response.id;

        // Wait for the sensor to reopen the socket, can be shortened probably
        tokio::time::sleep(Duration::from_secs_f32(1.0)).await;

        let response = self
            .post(host_uri.clone() + "pair/confirm")
            .header(PAIR_HEADER_NAME, id.as_str())
            .send_parse_err::<ErrorResponse>()
            .await?
            .map_err(|e| e.error.to_string())?;

        if response.is_success() {
            // Wait for the sensor to reopen the socket, can be shortened probably
            tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
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
            .send_parse_err::<ErrorResponse>()
            .await?
            .map_err(|e| e.error.to_string());

        let response = response?;

        if response.is_success() {
            // Wait for the sensor to reopen the socket, can be shortened probably
            tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
            let response = self
                .get(host_uri.clone() + "sensor")
                .send_parse::<SensorResponse, ErrorResponse>()
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
        count: Option<u64>,
        max_age: Option<u64>,
    ) -> Result<Vec<Measurement>, anyhow::Error>;
}

impl TempSensorService for reqwest::Client {
    async fn get_temp(
        &self,
        host: &str,
        count: Option<u64>,
        max_age: Option<u64>,
    ) -> Result<Vec<Measurement>, anyhow::Error> {
        let host_uri = format!("http://{}:{}/", host, SENSOR_PORT);
        let response = self
            .get(host_uri.clone() + "temp")
            .query(&[
                ("count", count),
                ("max_age", max_age),
            ])
            .send_parse::<MeasurementsResponse, ErrorResponse>()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .map_err(|e| anyhow::anyhow!("{}", e.error))?;

        Ok(response.measurements)
    }
}
