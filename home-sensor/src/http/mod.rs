use crate::{
    models::http::{Request, ResponseBuilder},
    storage::StoreProvider,
    BUTTON,
};
use core::{borrow::BorrowMut, cell::RefCell, str::FromStr};
use critical_section::Mutex;
use embedded_io::{Read, Write};
use esp_hal::macros::handler;
use esp_println::println;
use esp_storage::FlashStorage;
use esp_wifi::{current_millis, wifi::WifiStaDevice, wifi_interface::Socket};
use heapless::String;
use home_common::models::ErrorResponse;
use route::pair::CURRENT_KEYS;
use status::StatusCode;

pub mod route;
pub mod status;

pub const HEADERS_LEN: usize = 16;
pub const BUFFER_LEN: usize = 1024;
pub const RESPONSE_HEADER_LEN: usize = 512;
pub const RESPONSE_BODY_LEN: usize = 512;

pub struct Timeout {
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

pub static OPENED_TIMEOUT: Mutex<RefCell<Timeout>> = Mutex::new(RefCell::new(Timeout::new(30_000)));

pub fn server_loop(socket: &mut Socket<WifiStaDevice>) -> ! {
    log::info!("Start listening!");
    let mut disconnect_timeout = Timeout::new(100);
    let mut pairing = false;
    loop {
        socket.work();

        if !socket.is_open() {
            log::info!("Waiting for connection");
            socket.listen(home_common::consts::SENSOR_PORT).unwrap();
        }

        critical_section::with(|cs| {
            let timeout = OPENED_TIMEOUT.borrow_ref(cs);
            if !pairing && timeout.started() {
                pairing = true;
            }
            if pairing && timeout.finished() {
                log::info!("Timeout reached, closing connection");
                CURRENT_KEYS.borrow_ref_mut(cs).clear();
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
            let pair_confirm_route = route::pair::confirm();
            let pair_id = request
                .headers
                .get(home_common::consts::PAIR_HEADER_NAME)
                .and_then(|id| {
                    FlashStorage::new()
                        .get()
                        .ok()
                        .and_then(|s| s.paired_keys.iter().find(|k| k.as_str() == id).cloned())
                });
            let response = match (
                pairing,
                (pair_route.is_match)(&request),
                (pair_confirm_route.is_match)(&request),
            ) {
                (true, true, _) => (pair_route.response)(&request, pair_id),
                (true, false, true) => (pair_confirm_route.response)(&request, pair_id),
                (false, pr, pcr) if pr || pcr => ResponseBuilder::default()
                    .with_code(StatusCode::FORBIDDEN)
                    .with_data(&ErrorResponse {
                        error: "To connect use /pair endpoint and pairing button on the device."
                            .try_into()
                            .unwrap(),
                    })
                    .into(),
                _ => match route::routes().into_iter().find(|r| (r.is_match)(&request)) {
                    Some(route) => (route.response)(&request, pair_id),
                    None => ResponseBuilder::default()
                        .with_code(StatusCode::NOT_FOUND)
                        .with_data(&ErrorResponse {
                            error: String::from_str("Not found").unwrap(),
                        })
                        .into(),
                },
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
