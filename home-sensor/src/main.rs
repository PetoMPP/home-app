#![no_std]
#![no_main]

use embedded_io::{Read, Write};
use esp_backtrace as _;
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl};
use esp_println::logger::init_logger;
use heapless::{String, Vec};
use smoltcp::iface::SocketStorage;

mod led_button;
mod wifi;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    // Set clocks at maximum frequency
    let clocks = ClockControl::max(system.clock_control).freeze();

    init_logger(log::LevelFilter::Info);
    log::info!("Starting..");

    init_led_button!(peripherals);

    let wifi_builder = wifi_builder!(peripherals, clocks);
    let mut storage: [SocketStorage; 3] = Default::default();
    let wifi = wifi_builder.connect(&mut storage);

    let mut rx_buffer = [0u8; 2048];
    let mut tx_buffer = [0u8; 2048];
    let mut socket = wifi.get_socket(&mut rx_buffer, &mut tx_buffer);

    loop {
        if !socket.is_open() {
            log::info!("Listening on port {}", home_consts::SENSOR_PORT);
            socket.listen(home_consts::SENSOR_PORT).unwrap();
        }

        if !socket.is_connected() {
            log::warn!("Socket disconnected..");
            socket.close();
            continue;
        }
        log::info!("Connected!");
        log::info!("Reading message..");
        let mut req_buffer = [0u8; 2048];
        socket.read(&mut req_buffer).unwrap();
        log::info!("Received message: {:?}", String::from_utf8(Vec::<_, 2048>::from_slice(&req_buffer).unwrap()).unwrap());
        // not a valid http
        socket.write_fmt(format_args!("Hello from ESP32!")).unwrap();
        log::info!("Sent message");
        socket.close();
    }
}
