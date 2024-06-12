use super::Route;
use crate::models::{http::ResponseBuilder, json::PairData};
use core::cell::RefCell;
use critical_section::Mutex;
use heapless::{String, Vec};

pub const PAIR_HEADER_NAME: &str = "X-Pair-Id";
const PAIRED_KEY_LEN: usize = 64;
pub static mut PAIRED_KEYS: Mutex<RefCell<Vec<String<PAIRED_KEY_LEN>, 16>>> =
    Mutex::new(RefCell::new(Vec::new()));

pub fn pair() -> Route {
    Route {
        is_match: |r| r.method == "POST" && r.route == "/pair",
        response: |_| {
            let mut id = String::<PAIRED_KEY_LEN>::new();
            critical_section::with(|cs| {
                let mut ibuffer = itoa::Buffer::new();
                let mut rng = crate::RNG.borrow_ref_mut(cs).unwrap();
                loop {
                    if let Err(_) = id.push_str(ibuffer.format(rng.random())) {
                        break;
                    }
                }

                unsafe {
                    PAIRED_KEYS.borrow_ref_mut(cs).push(id.clone()).unwrap();
                }
            });

            ResponseBuilder::default()
                .with_data(&PairData { id: id.as_str() })
                .into()
        },
    }
}
