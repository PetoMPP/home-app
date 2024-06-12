use super::Route;
use crate::{
    http::{status::StatusCode, OPENED_TIMEOUT},
    models::{http::ResponseBuilder, Sensor},
    storage::StoreProvider,
};
use esp_storage::FlashStorage;
use heapless::String;
use home_common::models::SensorResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SensorDto {
    pub name: Option<String<64>>,
    pub location: Option<String<64>>,
    pub features: Option<u32>,
}

impl SensorDto {
    pub fn merge(self, sensor: Sensor) -> Sensor {
        Sensor {
            name: self.name.unwrap_or(sensor.name),
            location: self.location.unwrap_or(sensor.location),
            features: self.features.unwrap_or(sensor.features),
        }
    }
}

pub fn get() -> Route {
    Route {
        is_match: |r| r.method == "GET" && r.route == "/sensor",
        response: |_r| {
            let Ok((store, usage)) = FlashStorage::new().get_with_usage() else {
                return ResponseBuilder::<'_, usize>::default()
                    .with_code(StatusCode::INTERNAL_SERVER_ERROR)
                    .into();
            };

            let mut pairing = false;
            critical_section::with(|cs| {
                let timeout = OPENED_TIMEOUT.borrow(cs).borrow();
                if timeout.started() && !timeout.finished() {
                    pairing = true;
                }
            });

            let sensor = store.sensor;
            let sensor = SensorResponse {
                name: sensor.name,
                location: sensor.location,
                features: sensor.features,
                pairing,
                paired_keys: store.paired_keys.len() as u32,
                usage,
            };

            ResponseBuilder::default().with_data(&sensor).into()
        },
    }
}

pub fn post() -> Route {
    Route {
        is_match: |r| r.method == "POST" && r.route == "/sensor",
        response: |r| {
            let mut flash_storage = FlashStorage::new();
            let Ok(store) = flash_storage.get() else {
                return ResponseBuilder::<'_, usize>::default()
                    .with_code(StatusCode::INTERNAL_SERVER_ERROR)
                    .into();
            };
            let Ok(data) = r.body::<SensorDto>() else {
                return ResponseBuilder::<'_, usize>::default()
                    .with_code(StatusCode::BAD_REQUEST)
                    .into();
            };
            let mut store = store;
            store.sensor = data.merge(store.sensor);
            flash_storage.set(store);
            ResponseBuilder::<'_, usize>::default().into()
        },
    }
}
