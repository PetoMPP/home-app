#![no_std]
#![no_main]

use core::{cell::RefCell, ops::Not};
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
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
    let _clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    println!("Starting..");

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    // Set the interrupt handler for GPIO interrupts.
    io.set_interrupt_handler(handler);
    let led = Output::new(io.pins.gpio7, Level::Low);
    let mut button = Input::new(io.pins.gpio6, Pull::Up);

    critical_section::with(|cs| {
        // Assign handler and RefCells with the GPIO instances.
        button.listen(Event::AnyEdge);
        BUTTON.borrow_ref_mut(cs).replace(button);
        LED.borrow_ref_mut(cs).replace(led);
    });

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
