pub mod db {
    #[derive(Debug, Clone)]
    pub struct SensorEntity {
        pub name: String,
        pub location: String,
        pub features: u32,
        pub pair_id: String,
    }
}