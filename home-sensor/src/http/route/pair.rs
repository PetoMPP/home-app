use core::cell::RefCell;
use super::{headers, Route};
use crate::http::status::StatusCode;
use critical_section::Mutex;
use heapless::{FnvIndexMap, String, Vec};

pub const PAIR_HEADER_NAME: &str = "X-Pair-Id";
const PAIRED_KEY_LEN: usize = 64;
pub static mut PAIRED_KEYS: Mutex<RefCell<Vec<String<PAIRED_KEY_LEN>, 16>>> = Mutex::new(RefCell::new(Vec::new()));

pub fn pair() -> Route {
    Route {
        is_match: |r| r.method == "POST" && r.route == "/pair",
        response: |_| {
            let mut ibuffer = itoa::Buffer::new();
            let mut b = Vec::<_, 512>::new();
            critical_section::with(|cs| {
                let mut ibuffer = itoa::Buffer::new();
                let mut rng = crate::RNG.borrow_ref_mut(cs).unwrap();
                let mut id = String::<PAIRED_KEY_LEN>::new();
                loop {
                    if let Err(_) = id.push_str(ibuffer.format(rng.random())) {
                        break;
                    }
                }
                unsafe {
                    PAIRED_KEYS.borrow_ref_mut(cs).push(id.clone()).unwrap();
                }
                b.extend_from_slice(b"{\"uuid\": \"").unwrap();
                b.extend_from_slice(id.as_bytes()).unwrap();
                b.extend_from_slice(b"\"}").unwrap();
            });

            let mut h = FnvIndexMap::new();
            h.insert("Content-Type", "application/json").unwrap();
            h.insert("Content-Length", ibuffer.format(b.len())).unwrap();
            let mut response = headers(StatusCode::OK, &h);
            response.extend_from_slice(b.as_slice()).unwrap();
            response
        },
    }
}
