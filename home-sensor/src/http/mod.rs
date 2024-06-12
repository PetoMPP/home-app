use embedded_io::{Read, Write};
use esp_println::println;
use esp_wifi::{current_millis, wifi::WifiStaDevice, wifi_interface::Socket};
use heapless::FnvIndexMap;
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
    pub fn new(delay_ms: u64) -> Self {
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
        current_millis() > self.end_time.unwrap_or(u64::MAX)
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

pub fn server_loop<'s, 'r>(socket: &'s mut Socket<WifiStaDevice>) -> ! {
    log::info!("Start listening!");
    let mut disconnect_timeout = Timeout::new(100);
    loop {
        socket.work();

        if !socket.is_open() {
            log::info!("Waiting for connection");
            socket.listen(home_consts::SENSOR_PORT).unwrap();
        }

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

            let nf = route::not_found();
            let routes = route::routes();
            let response = routes
                .into_iter()
                .find(|r| (r.is_match)(&request))
                .unwrap_or(nf);

            socket
                .write_all((response.response)(&request).as_slice())
                .unwrap();

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
