use super::Route;
use crate::{http::status::StatusCode, models::http::ResponseBuilder, storage::StoreProvider};
use core::{cell::RefCell, str::FromStr};
use critical_section::Mutex;
use esp_storage::FlashStorage;
use heapless::{String, Vec};
use home_common::models::{ErrorResponse, PairResponse};

pub static CURRENT_KEYS: Mutex<RefCell<Vec<String<32>, 16>>> = Mutex::new(RefCell::new(Vec::new()));

pub fn clear_keys() {
    critical_section::with(|cs| CURRENT_KEYS.borrow_ref_mut(cs).clear());
}

pub fn pair() -> Route {
    Route {
        is_match: |r| r.method == "POST" && r.route == "/pair",
        response: |_| {
            let mut id = Ok(new_id());
            critical_section::with(|cs| {
                if let Err(eid) = CURRENT_KEYS.borrow_ref_mut(cs).push(id.clone().unwrap()) {
                    id = Err(eid);
                }
            });
            let Ok(id) = id else {
                return ResponseBuilder::default()
                    .with_code(StatusCode::TOO_MANY_REQUESTS)
                    .with_data(&ErrorResponse {
                        error: String::from_str("Too many pairing requests").unwrap(),
                    })
                    .into();
            };

            ResponseBuilder::default()
                .with_data(&PairResponse { id })
                .into()
        },
    }
}

pub fn confirm() -> Route {
    Route {
        is_match: |r| r.method == "POST" && r.route == "/pair/confirm",
        response: |r| {
            let Some(id) = r
                .headers
                .get(home_common::consts::PAIR_HEADER_NAME)
                .and_then(|id| String::from_str(id).ok())
            else {
                return ResponseBuilder::<'_, usize>::default()
                    .with_code(StatusCode::BAD_REQUEST)
                    .into();
            };

            let mut valid = false;
            critical_section::with(|cs| {
                valid = CURRENT_KEYS.borrow_ref(cs).iter().any(|k| k == &id)
            });

            if !valid {
                return ResponseBuilder::default()
                    .with_code(StatusCode::NOT_FOUND)
                    .with_data(&ErrorResponse {
                        error: String::from_str("Pairing key not found").unwrap(),
                    })
                    .into();
            }

            let flash_storage = &mut FlashStorage::new();
            let Ok(mut store) = flash_storage.get() else {
                return ResponseBuilder::<'_, usize>::default()
                    .with_code(StatusCode::INTERNAL_SERVER_ERROR)
                    .into();
            };
            let Ok(_) = store.paired_keys.insert(id) else {
                return ResponseBuilder::default()
                    .with_code(StatusCode::INSUFFICIENT_STORAGE)
                    .with_data(&ErrorResponse {
                        error: String::from_str("Paired devices storage full").unwrap(),
                    })
                    .into();
            };
            flash_storage.set(store);

            ResponseBuilder::<'_, usize>::default().into()
        },
    }
}

fn new_id() -> String<32> {
    let mut id = String::new();
    critical_section::with(|cs| {
        let mut input = [0u8; 16];
        let mut output = [0u8; 16 * 2];
        let mut rng = crate::RNG.borrow_ref_mut(cs).unwrap();
        rng.read(&mut input);
        hex::encode_to_slice(&mut input, &mut output).unwrap();
        id = String::from_str(unsafe { core::str::from_utf8_unchecked(&output) }).unwrap();
    });

    id
}
