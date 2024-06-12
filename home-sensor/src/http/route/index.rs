use heapless::{FnvIndexMap, Vec};
use crate::http::status::StatusCode;
use super::{headers, Route};

pub const fn index() -> Route {
    Route {
        is_match: |r| r.method == "GET" && r.route == "/",
        response: |r| {
            let h = headers(StatusCode::OK, &FnvIndexMap::new());
            let mut response = Vec::new();
            response.extend_from_slice(h.as_bytes()).unwrap();
            response.extend_from_slice(r.body.as_bytes()).unwrap();
            response
        },
    }
}
