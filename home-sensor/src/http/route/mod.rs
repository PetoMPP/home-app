use crate::models::http::{Request, Response};
use heapless::Vec;

pub mod pair;

mod sensor;

#[derive(Debug)]
pub struct Route {
    pub is_match: fn(&Request) -> bool,
    pub response: fn(&Request, bool) -> Response,
}

pub fn routes() -> Vec<Route, 16> {
    let mut routes = Vec::new();
    routes.push(sensor::get()).unwrap();
    routes.push(sensor::post()).unwrap();
    routes
}
