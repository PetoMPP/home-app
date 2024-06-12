use crate::BUTTON;
use core::{borrow::BorrowMut, cell::RefCell};
use critical_section::Mutex;
use embedded_io::{Read, Write};
use esp_hal::macros::handler;
use esp_println::println;
use esp_wifi::{current_millis, wifi::WifiStaDevice, wifi_interface::Socket};
use heapless::FnvIndexMap;
use route::headers;
use status::StatusCode;

pub mod route;

mod status;

pub const HEADERS_LEN: usize = 16;
pub const BUFFER_LEN: usize = 1024;
pub const RESPONSE_HEADER_LEN: usize = 512;
pub const RESPONSE_BODY_LEN: usize = 512;

struct Timeout {
    delay_ms: u64,
    end_time: Option<u64>,
}

impl Timeout {
    pub const fn new(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            end_time: None,
        }
    }

    pub fn start(&mut self) {
        self.end_time = Some(self.delay_ms + current_millis());
    }

    pub fn reset(&mut self) {
        self.end_time = None;
    }

    pub fn finished(&self) -> bool {
        current_millis() > self.end_time.unwrap_or(0)
    }

    pub fn started(&self) -> bool {
        self.end_time.is_some()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Request<'r> {
    route: &'r str,
    method: &'r str,
    headers: FnvIndexMap<&'r str, &'r str, HEADERS_LEN>,
    body: &'r str,
}

impl<'r> TryFrom<&'r [u8]> for Request<'r> {
    type Error = StatusCode;

    fn try_from(value: &'r [u8]) -> Result<Self, Self::Error> {
        let value = unsafe { core::str::from_utf8_unchecked(&value) };
        let Some(he) = value.find("\r\n\r\n") else {
            return Err(StatusCode::BAD_REQUEST);
        };
        let header_str = &value[..he];
        let mut lines = header_str.lines();
        let path_line = lines.next().ok_or(StatusCode::BAD_REQUEST)?;
        let me = path_line.find(" ").ok_or(StatusCode::BAD_REQUEST)?;
        let method = &path_line[..me];
        let path_line = &path_line[me + 1..];
        let route = &path_line[..path_line.find(" ").ok_or(StatusCode::BAD_REQUEST)?];
        let mut headers = FnvIndexMap::new();
        for header in lines {
            let Some((key, value)) = header.split_once(": ") else {
                continue;
            };
            headers
                .insert(key, value)
                .map_err(|_| StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)?;
        }
        let body = &value[he + 4..];

        Ok(Request {
            route,
            method,
            headers,
            body,
        })
    }
}

static OPENED_TIMEOUT: Mutex<RefCell<Timeout>> = Mutex::new(RefCell::new(Timeout::new(15_000)));

pub fn server_loop<'s, 'r>(socket: &'s mut Socket<WifiStaDevice>) -> ! {
    log::info!("Start listening!");
    let mut disconnect_timeout = Timeout::new(100);
    let mut pairing = false;
    loop {
        socket.work();

        if !socket.is_open() {
            log::info!("Waiting for connection");
            socket.listen(home_consts::SENSOR_PORT).unwrap();
        }

        critical_section::with(|cs| {
            let timeout = OPENED_TIMEOUT.borrow_ref(cs);
            if !pairing && timeout.started() {
                pairing = true;
            }
            if pairing && timeout.finished() {
                log::info!("Timeout reached, closing connection");
                pairing = false;
            }
        });

        if socket.is_connected() {
            println!("Connected");

            // Start 10s timeout for socket read
            let mut socket_read_timeout = Timeout::new(10_000);
            socket_read_timeout.reset();
            socket_read_timeout.start();
            let mut buffer = [0u8; BUFFER_LEN];
            let mut pos = None;
            loop {
                let Ok(read_pos) = socket.read(&mut buffer) else {
                    if socket_read_timeout.finished() {
                        socket.close();
                        break;
                    }
                    continue;
                };
                pos = Some(read_pos);
                break;
            }
            if !socket.is_open() {
                continue;
            }

            let Some(pos) = pos else {
                continue;
            };

            let Ok(request) = Request::try_from(&buffer[..pos]) else {
                continue;
            };

            let pair_route = route::pair::pair();
            let response = match pairing {
                true if (pair_route.is_match)(&request) => (pair_route.response)(&request),
                _ => {
                    let mut valid = false;
                    if let Some(id) = request.headers.get(route::pair::PAIR_HEADER_NAME) {
                        critical_section::with(|cs| {
                            let keys = unsafe { route::pair::PAIRED_KEYS.borrow_ref(cs) };
                            valid = keys.iter().any(|k| k.as_str() == *id);
                        });
                    }

                    match (
                        valid,
                        route::routes().into_iter().find(|r| (r.is_match)(&request)),
                    ) {
                        (true, Some(route)) => (route.response)(&request),
                        (true, None) => headers(StatusCode::NOT_FOUND, &Default::default()),
                        _ => {
                            let mut ibuffer = itoa::Buffer::new();
                            let b = b"{\"error\": \"To connect use /pair endpoint and pairing button on the device.\"}";
                            let mut h = FnvIndexMap::new();
                            h.insert("Content-Type", "application/json").unwrap();
                            h.insert("Content-Length", ibuffer.format(b.len())).unwrap();
                            let mut h = headers(StatusCode::FORBIDDEN, &h);
                            h.extend_from_slice(b).unwrap();
                            h
                        }
                    }
                }
            };

            socket.write_all(response.as_slice()).unwrap();

            socket.flush().unwrap();
            socket.close();

            // Start timeout for connection break
            // This is a workaround for inputs longer than the buffer
            disconnect_timeout.reset();
            disconnect_timeout.start();

            println!("Request handling done!");
        }

        if disconnect_timeout.finished() {
            println!("Aborting!!");
            socket.disconnect();
            disconnect_timeout.reset();
        }
    }
}

#[handler]
pub fn handler() {
    critical_section::with(|cs| {
        let mut but = BUTTON.borrow_ref_mut(cs);
        let but = but.as_mut().unwrap();
        let mut timeout = OPENED_TIMEOUT.borrow_ref_mut(cs);
        let timeout = timeout.borrow_mut();
        timeout.reset();
        timeout.start();
        but.clear_interrupt();
    });
}
