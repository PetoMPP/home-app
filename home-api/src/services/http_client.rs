use reqwest::{RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use std::error::Error;

pub trait HttpRequest {
    async fn send_parse<T, E>(self) -> Result<Result<T, E>, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        E: DeserializeOwned;

    async fn send_parse_err<E>(self) -> Result<Result<StatusCode, E>, Box<dyn Error + Send + Sync>>
    where
        E: DeserializeOwned;

    async fn send_parse_retry<T, E>(
        self,
        retries: usize,
    ) -> Result<Result<T, E>, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        E: DeserializeOwned;

    async fn send_parse_err_retry<E>(
        self,
        retries: usize,
    ) -> Result<Result<StatusCode, E>, Box<dyn Error + Send + Sync>>
    where
        E: DeserializeOwned;
}

impl HttpRequest for RequestBuilder {
    async fn send_parse<T, E>(self) -> Result<Result<T, E>, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        E: DeserializeOwned,
    {
        let resp = self.send().await?;
        let bytes = resp.bytes().await?;

        match serde_json::from_slice::<T>(&bytes) {
            Ok(result) => Ok(Ok(result)),
            Err(_) => Ok(Err(serde_json::from_slice::<E>(&bytes)?)),
        }
    }

    async fn send_parse_err<E>(self) -> Result<Result<StatusCode, E>, Box<dyn Error + Send + Sync>>
    where
        E: DeserializeOwned,
    {
        let resp = self.send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        match serde_json::from_slice::<E>(&bytes) {
            Ok(result) => Ok(Err(result)),
            Err(_) => Ok(Ok(status)),
        }
    }

    async fn send_parse_retry<T, E>(
        self,
        retries: usize,
    ) -> Result<Result<T, E>, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
        E: DeserializeOwned,
    {
        let retries = retries.max(1);
        let mut attempts = 0;
        let mut result = self
            .try_clone()
            .ok_or(anyhow::anyhow!("Unable to clone request"))?
            .send_parse::<T, E>()
            .await;
        while let Err(_) = result {
            attempts += 1;
            if attempts >= retries {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs_f32(0.2)).await;
            result = self.try_clone().unwrap().send_parse::<T, E>().await;
        }

        result
    }

    async fn send_parse_err_retry<E>(
        self,
        retries: usize,
    ) -> Result<Result<StatusCode, E>, Box<dyn Error + Send + Sync>>
    where
        E: DeserializeOwned,
    {
        let retries = retries.max(1);
        let mut attempts = 0;
        let mut result = self
            .try_clone()
            .ok_or(anyhow::anyhow!("Unable to clone request"))?
            .send_parse_err::<E>()
            .await;
        while let Err(_) = result {
            attempts += 1;
            if attempts >= retries {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs_f32(0.2)).await;
            result = self.try_clone().unwrap().send_parse_err::<E>().await;
        }

        result
    }
}
