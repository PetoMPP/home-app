use super::http_client::HttpRequest;
use crate::models::db::SensorEntity;
use home_common::models::{ErrorResponse, PairResponse, Sensor, SensorDto, SensorResponse};
use std::{error::Error, time::Duration};

pub trait SensorService {
    async fn get_sensor(&self, host: &str) -> Result<SensorEntity, Box<dyn Error>>;
    async fn pair(&self, host: &str) -> Result<SensorEntity, Box<dyn Error>>;
    async fn update_sensor(
        &self,
        host: &str,
        pair_id: &str,
        sensor: Sensor,
    ) -> Result<Sensor, Box<dyn Error>>;
}

impl SensorService for reqwest::Client {
    async fn get_sensor(&self, host: &str) -> Result<SensorEntity, Box<dyn Error>> {
        let host_uri = format!("http://{}:{}/", host, home_common::consts::SENSOR_PORT);
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
            features: response.features,
            host: host.to_string(),
            pair_id: None,
        };

        Ok(sensor_entity)
    }

    async fn pair(&self, host: &str) -> Result<SensorEntity, Box<dyn Error>> {
        let host_uri = format!("http://{}:{}/", host, home_common::consts::SENSOR_PORT);
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
            .header(home_common::consts::PAIR_HEADER_NAME, id.as_str())
            .send_parse_err::<ErrorResponse>()
            .await?
            .map_err(|e| e.error.to_string())?;

        if response.is_success() {
            // Wait for the sensor to reopen the socket, can be shortened probably
            tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
            let mut sensor = self.get_sensor(host).await?;
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
        sensor: Sensor,
    ) -> Result<Sensor, Box<dyn Error>> {
        let host_uri = format!("http://{}:{}/", host, home_common::consts::SENSOR_PORT);
        let sensor_dto = sensor.into();
        let response = self
            .post(host_uri.clone() + "sensor")
            .header(home_common::consts::PAIR_HEADER_NAME, pair_id)
            .json::<SensorDto>(&sensor_dto)
            .send_parse_err::<ErrorResponse>()
            .await?
            .map_err(|e| e.error.to_string());

        println!("{:?}", response);
        let response = response?;

        if response.is_success() {
            // Wait for the sensor to reopen the socket, can be shortened probably
            tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
            let response = self
                .get(host_uri.clone() + "sensor")
                .send_parse::<SensorResponse, ErrorResponse>()
                .await?
                .map_err(|e| e.error.to_string())?;

            return Ok(response.into());
        }

        Err("Update failed".into())
    }
}
