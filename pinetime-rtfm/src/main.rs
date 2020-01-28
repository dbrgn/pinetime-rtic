#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

#[allow(unused_imports)]
use nrf52832_hal as hal;

#[entry]
fn main() -> ! {
    hprintln!("Hello PineTime from Rust!").unwrap();

    loop {}
}
