#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nrf52832_hal as hal;
use nrf52832_hal::gpio::Level;
use nrf52832_hal::prelude::*;

#[entry]
fn main() -> ! {
    //hprintln!("Hello PineTime from Rust!").unwrap();

    let p = cortex_m::Peripherals::take().unwrap();
    let dp = hal::nrf52832_pac::Peripherals::take().unwrap();

    let gpio = dp.P0.split();

    let mut backlight_low = gpio.p0_14.into_push_pull_output(Level::High);
    let mut backlight_mid = gpio.p0_22.into_push_pull_output(Level::High);
    let mut backlight_high = gpio.p0_23.into_push_pull_output(Level::High);
    backlight_low.set_low();
    backlight_mid.set_low();
    backlight_high.set_low();

    loop {
        asm::nop();
    }
}
