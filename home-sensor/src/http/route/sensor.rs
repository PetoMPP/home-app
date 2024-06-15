use super::Route;
use crate::{
    http::{status::StatusCode, OPENED_TIMEOUT},
    models::http::ResponseBuilder,
    storage::StoreProvider,
};
use esp_storage::FlashStorage;
use home_common::models::{SensorDto, SensorResponse};

pub fn get() -> Route {
    Route {
        is_match: |r| r.method == "GET" && r.route == "/sensor",
        response: |_r, _paired| {
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
        response: |r, paired| {
            if !paired {
                return ResponseBuilder::<'_, usize>::default()
                    .with_code(StatusCode::FORBIDDEN)
                    .into();
            }
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
