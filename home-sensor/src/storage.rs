use crate::models::storage::Store;
use embedded_storage::{ReadStorage, Storage};
use esp_storage::FlashStorage;
use home_common::models::StoreUsage;

pub trait StoreProvider {
    fn init(&mut self);
    fn get(&mut self) -> Result<Store, serde_json_core::de::Error> {
        self.get_with_usage().map(|(store, _)| store)
    }
    fn get_with_usage(&mut self) -> Result<(Store, StoreUsage), serde_json_core::de::Error>;
    fn set(&mut self, store: Store);
}

const OFFSET: u32 = 0x9000;
const CAPACITY: u32 = 0x6000;

impl StoreProvider for FlashStorage {
    fn init(&mut self) {
        if let Err(_) = self.get() {
            log::info!("No data found in storage, initializing..");
            self.set(Store::default());
        }
    }

    fn get_with_usage(&mut self) -> Result<(Store, StoreUsage), serde_json_core::de::Error> {
        let mut buffer = [0u8; CAPACITY as usize];
        log::info!("Reading storage");
        self.read(OFFSET, &mut buffer).unwrap();
        let len = buffer
            .iter()
            .position(|&x| x == 0xFF)
            .unwrap_or(CAPACITY as usize);
        let (store, _) = serde_json_core::from_slice::<'_, Store>(&buffer[..len])?;
        Ok((
            store,
            StoreUsage {
                used: len as u32,
                total: CAPACITY,
            },
        ))
    }

    fn set(&mut self, store: Store) {
        log::info!("Writing storage: {:?}", self.capacity());
        let mut buffer = [0u8; CAPACITY as usize];
        let pos = serde_json_core::to_slice(&store, &mut buffer).unwrap();
        buffer[pos..].fill(0xFF); // Fill the rest with 0xFF
        self.write(OFFSET, &buffer).unwrap();
    }
}
