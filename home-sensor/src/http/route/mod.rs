use super::{status::StatusCode, Request, HEADERS_LEN, RESPONSE_BODY_LEN, RESPONSE_HEADER_LEN};
use heapless::{FnvIndexMap, String, Vec};
use index::index;

pub mod index;

#[derive(Debug)]
pub struct Route {
    pub is_match: fn(&Request) -> bool,
    pub response: fn(&Request) -> Vec<u8, RESPONSE_BODY_LEN>,
}

pub fn headers(
    code: StatusCode,
    headers: &FnvIndexMap<&str, &str, HEADERS_LEN>,
) -> String<RESPONSE_HEADER_LEN> {
    let mut header_str = String::new();
    header_str.push_str(&code.http_header()).unwrap();
    for (k, v) in headers {
        header_str.push_str(k).unwrap();
        header_str.push_str(": ").unwrap();
        header_str.push_str(v).unwrap();
        header_str.push_str("\r\n").unwrap();
    }
    header_str.push_str("\r\n").unwrap();
    header_str
}

pub const fn not_found() -> Route {
    Route {
        is_match: |_| true,
        response: |_| {
            let mut vec = Vec::new();
            vec.extend_from_slice(headers(StatusCode::NOT_FOUND, &FnvIndexMap::new()).as_bytes())
                .unwrap();
            vec.extend_from_slice("{ \"error\": 404 }".as_bytes())
                .unwrap();
            vec
        },
    }
}

pub fn routes() -> Vec<Route, 16> {
    let mut routes = Vec::new();
    routes.push(index()).unwrap();
    routes
}
