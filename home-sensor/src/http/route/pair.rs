use super::Route;
use crate::{http::status::StatusCode, models::http::ResponseBuilder, storage::StoreProvider};
use esp_storage::FlashStorage;
use heapless::String;
use home_common::models::PairResponse;

pub const PAIR_HEADER_NAME: &str = "X-Pair-Id";
const PAIRED_KEY_LEN: usize = 64;

pub fn pair() -> Route {
    Route {
        is_match: |r| r.method == "POST" && r.route == "/pair",
        response: |_| {
            let flash_storage = &mut FlashStorage::new();
            let Ok(mut store) = flash_storage.get() else {
                return ResponseBuilder::<'_, usize>::default()
                    .with_code(StatusCode::INTERNAL_SERVER_ERROR)
                    .into();
            };
            let mut id = String::<PAIRED_KEY_LEN>::new();
            critical_section::with(|cs| {
                let mut ibuffer = itoa::Buffer::new();
                let mut rng = crate::RNG.borrow_ref_mut(cs).unwrap();
                loop {
                    if let Err(_) = id.push_str(ibuffer.format(rng.random())) {
                        break;
                    }
                }
            });
            store.paired_keys.push(id.clone()).unwrap();
            flash_storage.set(store);

            ResponseBuilder::default()
                .with_data(&PairResponse { id: id.as_str() })
                .into()
        },
    }
}
