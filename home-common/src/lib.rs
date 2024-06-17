#![no_std]
pub mod consts {
    pub const SENSOR_PORT: u16 = 42069;
    pub const PAIR_HEADER_NAME: &str = "X-Pair-Id";
}

pub mod models {
    use heapless::String;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct ErrorResponse {
        pub error: String<256>,
    }

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct PairResponse {
        pub id: String<32>,
    }

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct Sensor {
        pub name: String<64>,
        pub location: String<64>,
        pub features: u32,
    }

    impl From<SensorResponse> for Sensor {
        fn from(sensor: SensorResponse) -> Self {
            Sensor {
                name: sensor.name,
                location: sensor.location,
                features: sensor.features,
            }
        }
    }

    impl From<Sensor> for SensorDto {
        fn from(val: Sensor) -> Self {
            SensorDto {
                name: Some(val.name),
                location: Some(val.location),
                features: Some(val.features),
            }
        }
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
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

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct SensorResponse {
        pub name: String<64>,
        pub location: String<64>,
        pub features: u32,
        pub pairing: bool,
        pub paired_keys: u32,
        pub usage: StoreUsage,
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct StoreUsage {
        pub used: u32,
        pub total: u32,
    }

    impl StoreUsage {
        pub fn percent(&self) -> f32 {
            self.used as f32 * 100.0 / self.total as f32
        }
    }
}

pub mod prelude {
    pub use crate::*;
}
