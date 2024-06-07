#![no_std]
#![no_main]

use core::{cell::RefCell, ops::Not};
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Event, Gpio6, Gpio7, Input, Io, Level, Output, Pull},
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    system::SystemControl,
    timer::systimer::SystemTimer,
};
use esp_println::logger::init_logger;
use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi::{AccessPointInfo, AuthMethod, ClientConfiguration, Configuration};
use esp_wifi::wifi::{WifiError, WifiStaDevice};
use esp_wifi::wifi_interface::WifiStack;
use esp_wifi::{current_millis, initialize, EspWifiInitFor};
use smoltcp::iface::SocketStorage;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");

static BUTTON: Mutex<RefCell<Option<Input<Gpio6>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Output<Gpio7>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    // Set clocks at maximum frequency
    let clocks = ClockControl::max(system.clock_control).freeze();

    init_logger(log::LevelFilter::Info);
    log::info!("Starting..");

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    // Set the interrupt handler for GPIO interrupts.
    io.set_interrupt_handler(handler);
    let led = Output::new(io.pins.gpio7, Level::Low);
    let mut button = Input::new(io.pins.gpio6, Pull::Up);

    // Async section for button and led handling.
    critical_section::with(|cs| {
        // Assign handler and RefCells with the GPIO instances.
        button.listen(Event::AnyEdge);
        BUTTON.borrow_ref_mut(cs).replace(button);
        LED.borrow_ref_mut(cs).replace(led);
    });

    // Initialize the timers used for Wifi
    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();
    // Configure Wifi
    let wifi = peripherals.WIFI;
    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let (iface, device, mut controller, sockets) =
        create_network_interface(&init, wifi, WifiStaDevice, &mut socket_set_entries).unwrap();

    let mut auth_method = AuthMethod::WPA2Personal;
    if PASSWORD.is_empty() {
        auth_method = AuthMethod::None;
    }

    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        auth_method,
        ..Default::default()
    });

    let res = controller.set_configuration(&client_config);
    log::info!("Wi-Fi set_configuration returned {:?}", res);

    controller.start().unwrap();
    log::info!("Is wifi started: {:?}", controller.is_started());

    log::info!("Start Wifi Scan");
    let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> = controller.scan_n();
    if let Ok((res, _count)) = res {
        for ap in res {
            log::info!("{:?}", ap);
        }
    }

    log::info!("{:?}", controller.get_capabilities());
    log::info!("Wi-Fi connect: {:?}", controller.connect());

    log::info!("Wait to get connected");
    let delay = Delay::new(&clocks);
    const RETRY_DELAY_MS: u32 = 15000;
    loop {
        let res = controller.is_connected();
        match res {
            Ok(connected) => {
                if connected {
                    break;
                }
            }
            Err(err) => {
                log::warn!("{:?}, retry in {}s..", err, RETRY_DELAY_MS / 1000);
                delay.delay_millis(RETRY_DELAY_MS);
                log::info!("Wi-Fi reconnect: {:?}", controller.connect());
            }
        }
    }
    log::info!("{:?}", controller.is_connected());

    let wifi_stack = WifiStack::new(iface, device, sockets, current_millis);
    log::info!("Wait to get an ip address");
    loop {
        wifi_stack.work();

        if wifi_stack.is_iface_up() {
            log::info!("got ip {:?}", wifi_stack.get_ip_info());
            break;
        }
    }

    loop {}
}

#[handler]
fn handler() {
    critical_section::with(|cs| {
        let mut but = BUTTON.borrow_ref_mut(cs);
        let but = but.as_mut().unwrap();
        // Set the LED level to the opposite of the button level.
        LED.borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .set_level(Into::<bool>::into(but.get_level()).not().into());
        but.clear_interrupt();
    });
}
