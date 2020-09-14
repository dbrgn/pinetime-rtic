use embedded_hal::digital::v2::OutputPin;
use nrf52832_hal::gpio::{Output, Pin, PushPull};
use rtt_target::rprintln;

/// Control the backlight.
///
/// There are three active-low backlight pins, each connected to a FET that
/// toggles backlight power through a resistor.
///
/// - Low: 2.2 kΩ
/// - Mid: 100 Ω
/// - High: 30 Ω
///
/// Through combinations of these pins, 7 brightness levels (+ off) can be
/// configured.
pub struct Backlight {
    low: Pin<Output<PushPull>>,
    mid: Pin<Output<PushPull>>,
    high: Pin<Output<PushPull>>,

    /// The current brightness level (value between 0 and 7).
    brightness: u8,
}

impl Backlight {
    /// Initialize the backlight with the specified level (0–7).
    pub fn init(
        low: Pin<Output<PushPull>>,
        mid: Pin<Output<PushPull>>,
        high: Pin<Output<PushPull>>,
        brightness: u8,
    ) -> Self {
        let mut backlight = Self {
            low,
            mid,
            high,
            brightness,
        };
        backlight.set(brightness);
        backlight
    }

    /// Set the brightness level. Must be a value between 0 (off) and 7 (max
    /// brightness). Higher values are clamped to 7.
    pub fn set(&mut self, mut brightness: u8) {
        if brightness > 7 {
            brightness = 7;
        }
        rprintln!("Setting backlight brightness to {}", brightness);
        if brightness & 0x01 > 0 {
            self.low.set_low().unwrap();
        } else {
            self.low.set_high().unwrap();
        }
        if brightness & 0x02 > 0 {
            self.mid.set_low().unwrap();
        } else {
            self.mid.set_high().unwrap();
        }
        if brightness & 0x04 > 0 {
            self.high.set_low().unwrap();
        } else {
            self.high.set_high().unwrap();
        }
        self.brightness = brightness;
    }

    /// Turn off the backlight.
    pub fn off(&mut self) {
        self.set(0);
    }

    /// Increase backlight brightness.
    pub fn brighter(&mut self) {
        self.set(self.brightness + 1);
    }

    /// Decrease backlight brightness.
    pub fn darker(&mut self) {
        self.set(self.brightness - 1);
    }

    /// Return the current brightness level (value between 0 and 7).
    pub fn get_brightness(&self) -> u8 {
        self.brightness
    }
}
