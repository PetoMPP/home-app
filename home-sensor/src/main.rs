#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl};
use esp_println::logger::init_logger;
use smoltcp::iface::SocketStorage;

mod http;
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

    http::server_loop(&mut socket);
}
