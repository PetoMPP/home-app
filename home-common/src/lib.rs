#![no_std]
pub mod consts {
    pub const SENSOR_PORT: u16 = 42069;
}

pub mod models {
    use heapless::String;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct ErrorResponse<'e> {
        pub error: &'e str,
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct PairResponse<'p> {
        pub id: &'p str,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct SensorResponse {
        pub name: String<64>,
        pub location: String<64>,
        pub features: u32,
        pub pairing: bool,
        pub paired_keys: u32,
        pub usage: StoreUsage,
    }

    #[derive(Default, Debug, Serialize, Deserialize)]
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
