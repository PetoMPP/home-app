use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Sensor {
    pub id: String,
    pub features: u32,
}
