#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, gpio::{Event, Gpio6, Gpio7, Input, Io, Level, Output, Pull}, peripherals::Peripherals, prelude::*, rng::Rng, system::SystemControl
};
use esp_println::logger::init_logger;
use smoltcp::iface::SocketStorage;

mod http;
mod wifi;

pub static BUTTON: Mutex<RefCell<Option<Input<Gpio6>>>> = Mutex::new(RefCell::new(None));
pub static LED: Mutex<RefCell<Option<Output<Gpio7>>>> = Mutex::new(RefCell::new(None));
pub static RNG: Mutex<RefCell<Option<Rng>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    // Set clocks at maximum frequency
    let clocks = ClockControl::max(system.clock_control).freeze();

    init_logger(log::LevelFilter::Info);
    log::info!("Starting..");
    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(http::handler);

    // Initialize with led on
    let led = Output::new(io.pins.gpio7, Level::High);
    let mut button = Input::new(io.pins.gpio6, Pull::Up);

    critical_section::with(|cs| {
        button.listen(Event::RisingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button);
        LED.borrow_ref_mut(cs).replace(led);
    });

    let wifi_builder = wifi_builder!(peripherals, clocks);
    let mut storage: [SocketStorage; 3] = Default::default();
    let wifi = wifi_builder.connect(&mut storage);

    let mut rx_buffer = [0u8; 2048];
    let mut tx_buffer = [0u8; 2048];
    let mut socket = wifi.get_socket(&mut rx_buffer, &mut tx_buffer);

    // Turn off led after wifi is connected
    critical_section::with(|cs| {
        LED.borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .set_level(Level::Low);
    });

    http::server_loop(&mut socket);
}
