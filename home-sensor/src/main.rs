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
    system::SystemControl,
};
use esp_println::println;

static BUTTON: Mutex<RefCell<Option<Input<Gpio6>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Output<Gpio7>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    println!("Hello world!");

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    // Set the interrupt handler for GPIO interrupts.
    io.set_interrupt_handler(handler);
    // Set GPIO7 as an output, and set its state high initially.
    let mut led = Output::new(io.pins.gpio7, Level::Low);

    // Set GPIO6 as an input
    let mut button = Input::new(io.pins.gpio6, Pull::Up);

    // ANCHOR: critical_section
    critical_section::with(|cs| {
        button.listen(Event::AnyEdge);
        BUTTON.borrow_ref_mut(cs).replace(button);
        LED.borrow_ref_mut(cs).replace(led);
    });
    // ANCHOR_END: critical_section

    // let delay = Delay::new(&clocks);
    loop {
        // led.toggle();
        // delay.delay_millis(500u32);
    }
}

#[handler]
fn handler() {
    critical_section::with(|cs| {
        let mut but = BUTTON.borrow_ref_mut(cs);
        let but = but.as_mut().unwrap();
        LED.borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .set_level(Into::<bool>::into(but.get_level()).not().into());
        but.clear_interrupt();
    });
}
