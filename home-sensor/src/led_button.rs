use core::{cell::RefCell, ops::Not};
use critical_section::Mutex;
use esp_hal::{
    gpio::{Event, Gpio6, Gpio7, Input, Io, Level, Output, Pull},
    peripherals::{GPIO, IO_MUX},
    prelude::*,
};

static BUTTON: Mutex<RefCell<Option<Input<Gpio6>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Output<Gpio7>>>> = Mutex::new(RefCell::new(None));

#[macro_export]
macro_rules! init_led_button {
    ($peripherals:expr) => {
        led_button::init($peripherals.GPIO, $peripherals.IO_MUX);
    };
}

pub fn init(gpio: GPIO, io_mux: IO_MUX) {
    let mut io = Io::new(gpio, io_mux);
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
