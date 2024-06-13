use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use std::error::Error;

pub trait HttpRequest {
    async fn send_parse<T, E>(self) -> Result<Result<T, E>, Box<dyn Error>>
    where
        T: DeserializeOwned,
        E: DeserializeOwned;
}

impl HttpRequest for RequestBuilder {
    async fn send_parse<T, E>(self) -> Result<Result<T, E>, Box<dyn Error>>
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
}
