//! Delay implementation using regular timers.
//!
//! This is done because RTFM takes ownership of SYST, and the nrf52-hal by
//! default also wants SYST for its Delay implementation.

use embedded_hal::blocking::delay::DelayUs;
use nrf52832_hal::{
    self as hal,
    pac,
    timer::Timer,
};

pub struct TimerDelay {
    timer: hal::Timer<pac::TIMER0>,
}

impl TimerDelay {
    pub fn new(timer0: pac::TIMER0) -> Self {
        Self {
            timer: Timer::new(timer0),
        }
    }
}

impl DelayUs<u32> for TimerDelay {
    fn delay_us(&mut self, us: u32) {
        // Currently the HAL timer is hardcoded at 1 MHz,
        // so 1 cycle = 1 µs.
        let cycles = us;
        self.timer.delay(cycles);
    }
}
