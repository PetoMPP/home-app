use super::Route;
use crate::models::{http::ResponseBuilder, json::SensorData};

pub const fn index() -> Route {
    Route {
        is_match: |r| r.method == "GET" && r.route == "/",
        response: |_r| {
            ResponseBuilder::default()
                .with_data(&SensorData {
                    name: "Temperature Sensor",
                    location: "Living Room",
                    pairing: false,
                })
                .into()
        },
    }
}
