use embedded_hal::digital::v2::InputPin;
use nrf52832_hal::gpio::{Floating, Input, Pin};

pub struct BatteryStatus {
    /// Pin P0.12: High = battery, Low = charging.
    pin_charge_indication: Pin<Input<Floating>>,

    charging: bool,
    percent: u8,
}

impl BatteryStatus {
    /// Initialize the battery status.
    pub fn init(pin_charge_indication: Pin<Input<Floating>>) -> Self {
        let charging = pin_charge_indication.is_low().unwrap();
        Self {
            pin_charge_indication,
            charging,
            percent: 37,
        }
    }

    /// Return whether the watch is currently charging.
    pub fn is_charging(&self) -> bool {
        self.charging
    }

    /// Return the current battery charge in percent (0–100).
    pub fn percent(&self) -> u8 {
        if self.percent > 100 {
            100
        } else {
            self.percent
        }
    }

    /// Update the current battery status by reading information from the
    /// hardware. Return whether or not the values changed.
    pub fn update(&mut self) -> bool {
        let mut changed = false;

        // Check charging status
        let charging = self.pin_charge_indication.is_low().unwrap();
        if charging != self.charging {
            self.charging = charging;
            changed = true;
        }

        changed
    }
}
