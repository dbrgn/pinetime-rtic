#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m::asm;
use cortex_m_rt::entry;
use nrf52832_hal::{self as hal, pac};
use nrf52832_hal::gpio::Level;
use nrf52832_hal::prelude::*;

#[entry]
fn main() -> ! {
    let _p = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let gpio = hal::gpio::p0::Parts::new(dp.P0);

    let mut backlight_low = gpio.p0_14.into_push_pull_output(Level::High);
    let mut backlight_mid = gpio.p0_22.into_push_pull_output(Level::High);
    let mut backlight_high = gpio.p0_23.into_push_pull_output(Level::High);
    backlight_low.set_low().unwrap();
    backlight_mid.set_low().unwrap();
    backlight_high.set_low().unwrap();

    loop {
        asm::nop();
    }
}
