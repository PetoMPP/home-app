use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Sensor {
    pub id: String,
    pub features: u32,
}
