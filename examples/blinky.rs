use std::{thread, time::Duration};

use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys::EspError;

fn main() -> Result<(), EspError> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("Failed to take peripherals");

    let mut red = PinDriver::output(peripherals.pins.gpio3)?;

    loop {
        log::info!("Blinky");

        red.toggle()?;
        thread::sleep(Duration::from_secs(1));
    }
}
