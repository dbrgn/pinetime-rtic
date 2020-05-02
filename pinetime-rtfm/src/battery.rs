use embedded_hal::adc::OneShot;
use embedded_hal::digital::v2::InputPin;
use nrf52832_hal::gpio::{p0, Floating, Input};
use nrf52832_hal::saadc::{Saadc, SaadcConfig};
use nrf52832_hal::target::SAADC;

pub struct BatteryStatus {
    /// Pin P0.12: High = battery, Low = charging.
    pin_charge_indication: p0::P0_12<Input<Floating>>,

    /// Pin P0.31: Voltage level
    pin_voltage: p0::P0_31<Input<Floating>>,

    /// SAADC peripheral
    saadc: Saadc,

    /// Charging state
    charging: bool,

    /// Battery voltage in 0.1 volts
    voltage: u8,
}

impl BatteryStatus {
    /// Initialize the battery status.
    pub fn init(
        pin_charge_indication: p0::P0_12<Input<Floating>>,
        mut pin_voltage: p0::P0_31<Input<Floating>>,
        #[allow(non_snake_case)] SAADC: SAADC,
    ) -> Self {
        // Get initial charging state
        let charging = pin_charge_indication.is_low().unwrap();

        // Get initial voltage
        let mut saadc = Saadc::new(SAADC, SaadcConfig::default());
        let voltage =
            Self::convert_adc_measurement(saadc.read(&mut pin_voltage).unwrap()).unwrap_or(0);

        Self {
            pin_charge_indication,
            pin_voltage,
            saadc,
            charging,
            voltage,
        }
    }

    /// Convert a raw ADC measurement into a battery voltage in 0.1 volts.
    fn convert_adc_measurement(raw_measurement: i16) -> Option<u8> {
        if raw_measurement < 0 {
            // What?
            return None;
        }
        let adc_val: u32 = (raw_measurement as u16).into(); // keep as 32bit for multiplication
        let battery_voltage: u32 = (adc_val * 2000) / 4965; // we multiply the ADC value by 2 * 1000 for mV and divide by (2 ^ 14 / 3.3V reference)
        Some((battery_voltage / 100) as u8)
    }

    /// Return whether the watch is currently charging.
    ///
    /// This returns the stored value. To fetch current data, call `update()` first.
    pub fn is_charging(&self) -> bool {
        self.charging
    }

    /// Return the current battery charge in percent (0–100).
    ///
    /// This returns the stored value. To fetch current data, call `update()` first.
    pub fn percent(&self) -> u8 {
        unimplemented!();
    }

    /// Return the current battery voltage in 0.1 volts.
    ///
    /// This returns the stored value. To fetch current data, call `update()` first.
    pub fn voltage(&self) -> u8 {
        self.voltage
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

        // Check voltage
        let voltage =
            Self::convert_adc_measurement(self.saadc.read(&mut self.pin_voltage).unwrap())
                .unwrap_or(0);
        if voltage != self.voltage {
            self.voltage = voltage;
            changed = true;
        }

        changed
    }
}
