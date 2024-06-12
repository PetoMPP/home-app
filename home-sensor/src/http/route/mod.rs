use crate::models::http::{Request, Response};
use heapless::Vec;

pub mod pair;

mod index;

#[derive(Debug)]
pub struct Route {
    pub is_match: fn(&Request) -> bool,
    pub response: fn(&Request) -> Response,
}

pub fn routes() -> Vec<Route, 16> {
    let mut routes = Vec::new();
    routes.push(index::index()).unwrap();
    routes.push(pair::pair()).unwrap();
    routes
}
