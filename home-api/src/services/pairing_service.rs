use super::http_client::HttpRequest;
use home_common::models::{ErrorResponse, PairResponse};
use std::{error::Error, time::Duration};

pub trait PairingService {
    async fn can_pair(&self, host: &str) -> bool;
    async fn pair(&self, host: &str) -> Result<heapless::String<32>, Box<dyn Error>>;
}

impl PairingService for reqwest::Client {
    async fn can_pair(&self, host: &str) -> bool {
        // We get an expected response from the server
        self.post(host.to_string() + "pair")
            .timeout(Duration::from_secs_f32(0.2))
            .send_parse::<PairResponse, ErrorResponse>()
            .await
            .is_ok()
    }

    async fn pair(&self, host: &str) -> Result<heapless::String<32>, Box<dyn Error>> {
        let response = self
            .post(host.to_string() + "pair")
            .timeout(Duration::from_secs_f32(0.2))
            .send_parse::<PairResponse, ErrorResponse>()
            .await?
            .map_err(|e| e.error.to_string())?;

        let id = response.id;

        // Wait for the sensor to reopen the socket, can be shortened probably
        tokio::time::sleep(Duration::from_secs_f32(1.0)).await;

        self.post(host.to_string() + "pair/confirm")
            .header(home_common::consts::PAIR_HEADER_NAME, id.as_str())
            .send_parse_err::<ErrorResponse>()
            .await?
            .map_err(|e| e.error.to_string())?;

        Ok(id)
    }
}
