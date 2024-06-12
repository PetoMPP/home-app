use super::{status::StatusCode, Request, HEADERS_LEN, RESPONSE_BODY_LEN, RESPONSE_HEADER_LEN};
use heapless::{FnvIndexMap, Vec};

pub mod index;
pub mod pair;

#[derive(Debug)]
pub struct Route {
    pub is_match: fn(&Request) -> bool,
    pub response: fn(&Request) -> Vec<u8, RESPONSE_BODY_LEN>,
}

pub fn headers(
    code: StatusCode,
    headers: &FnvIndexMap<&str, &str, HEADERS_LEN>,
) -> Vec<u8, RESPONSE_HEADER_LEN> {
    let mut header = Vec::new();
    header
        .extend_from_slice(&code.http_header().as_bytes())
        .unwrap();
    for (k, v) in headers {
        header.extend_from_slice(k.as_bytes()).unwrap();
        header.extend_from_slice(b": ").unwrap();
        header.extend_from_slice(v.as_bytes()).unwrap();
        header.extend_from_slice(b"\r\n").unwrap();
    }
    header.extend_from_slice(b"\r\n").unwrap();
    header
}

pub fn routes() -> Vec<Route, 16> {
    let mut routes = Vec::new();
    routes.push(index::index()).unwrap();
    routes.push(pair::pair()).unwrap();
    routes
}
