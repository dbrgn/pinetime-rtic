#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m::asm;
use cortex_m_rt::entry;
use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::rectangle::Rectangle;
use nrf52832_hal::{self as hal, pac};
use nrf52832_hal::gpio::Level;
use nrf52832_hal::prelude::*;
use st7735_lcd::{self, Orientation};

static LCD_WIDTH: i32 = 240;
static LCD_HEIGHT: i32 = 240;

#[entry]
fn main() -> ! {
    let p = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Set up clocks
    let _clocks = hal::clocks::Clocks::new(dp.CLOCK);

    // Set up GPIO peripheral
    let gpio = hal::gpio::p0::Parts::new(dp.P0);

    // Set up SPI pins
    let spi_clk = gpio.p0_02.into_push_pull_output(Level::Low).degrade();
    let spi_mosi = gpio.p0_03.into_push_pull_output(Level::Low).degrade();
    let spi_miso = gpio.p0_04.into_floating_input().degrade();
    let spi_pins = hal::spim::Pins {
        sck: spi_clk,
        miso: Some(spi_miso),
        mosi: Some(spi_mosi),
    };

    // Set up LCD pins
    // LCD_CS (P0.25): Chip select
    let mut lcd_cs = gpio.p0_25.into_push_pull_output(Level::Low);
    // LCD_RS (P0.18): Data/clock pin
    let lcd_dc = gpio.p0_18.into_push_pull_output(Level::Low);
    // LCD_RESET (P0.26): Display reset
    let lcd_rst = gpio.p0_26.into_push_pull_output(Level::Low);

    // Initialize SPI
    let spi = hal::Spim::new(
        dp.SPIM1,
        spi_pins,
        // Use SPI at 8MHz (the fastest clock available on the nRF52832)
        // because otherwise refreshing will be super slow.
        hal::spim::Frequency::M8,
        // SPI must be used in mode 3. Mode 0 (the default) won't work.
        hal::spim::MODE_3,
        0,
    );


    // Get delay provider
    let mut delay = hal::delay::Delay::new(p.SYST);

    // Chip select must be held low while driving the display. It must be high
    // when using other SPI devices on the same bus (such as external flash
    // storage) so that the display controller won't respond to the wrong
    // commands.
    lcd_cs.set_low().unwrap();

    // Initialize LCD
    let mut lcd = st7735_lcd::ST7735::new(spi, lcd_dc, lcd_rst, false, true);
    lcd.init(&mut delay).unwrap();
    lcd.set_orientation(&Orientation::Landscape).unwrap();

    // Draw something onto the LCD
    let black_backdrop = Rectangle::new(
        Coord::new(0, 0),
        Coord::new(LCD_WIDTH, LCD_HEIGHT),
    ).fill(Some(0b00010_00000_01000u16.into()));
    lcd.draw(black_backdrop.into_iter());
    let ferris = Image16BPP::new(include_bytes!("../ferris.raw"), 86, 64)
        .translate(Coord::new(40, 33));
    lcd.draw(ferris.into_iter());

    // Enable backlight
    let _backlight_low = gpio.p0_14.into_push_pull_output(Level::High);
    let _backlight_mid = gpio.p0_22.into_push_pull_output(Level::High);
    let mut backlight_high = gpio.p0_23.into_push_pull_output(Level::High);
    backlight_high.set_low().unwrap();

    loop {
        asm::nop();
    }
}
