use super::{headers, Route};
use crate::http::status::StatusCode;
use heapless::FnvIndexMap;

pub const fn index() -> Route {
    Route {
        is_match: |r| r.method == "GET" && r.route == "/",
        response: |_r| {
            let mut response = headers(StatusCode::OK, &FnvIndexMap::new());
            response.extend_from_slice(b"The boy is using paired key!").unwrap();
            response
        },
    }
}
