#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use cortex_m::asm;
use embedded_graphics::prelude::*;
use embedded_graphics::{
    image::{Image, ImageRawLE},
    pixelcolor::Rgb565,
    primitives::rectangle::Rectangle,
    style::PrimitiveStyleBuilder,
};
use nrf52832_hal::{self as hal};
use nrf52832_hal::gpio::Level;
use nrf52832_hal::prelude::*;
use rtfm::app;
use st7789::{self, Orientation};

static LCD_WIDTH: u16 = 240;
static LCD_HEIGHT: u16 = 240;

#[app(device = nrf52832_hal::pac, peripherals = true)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        let p = cx.core;
        let dp = cx.device;

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
        let delay = hal::delay::Delay::new(p.SYST);

        // Chip select must be held low while driving the display. It must be high
        // when using other SPI devices on the same bus (such as external flash
        // storage) so that the display controller won't respond to the wrong
        // commands.
        lcd_cs.set_low().unwrap();

        // Initialize LCD
        let mut lcd = st7789::ST7789::new(spi, lcd_dc, lcd_rst, LCD_WIDTH, LCD_HEIGHT, delay);
        lcd.init().unwrap();
        lcd.set_orientation(&Orientation::Portrait).unwrap();

        // Enable backlight
        let _backlight_low = gpio.p0_14.into_push_pull_output(Level::High);
        let _backlight_mid = gpio.p0_22.into_push_pull_output(Level::High);
        let mut backlight_high = gpio.p0_23.into_push_pull_output(Level::High);
        backlight_high.set_low().unwrap();

        // Draw something onto the LCD
        let backdrop_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::GREEN)
            .build();
        let backdrop = Rectangle::new(
            Point::new(0, 0),
            Point::new(LCD_WIDTH as i32, LCD_HEIGHT as i32),
        ).into_styled(backdrop_style);
        backdrop.draw(&mut lcd).unwrap();
        let ferris_data = ImageRawLE::new(include_bytes!("../ferris.raw"), 86, 64);
        let ferris = Image::new(&ferris_data, Point::new(100, 80));
        ferris.draw(&mut lcd).unwrap();

        loop {
            asm::nop();
        }
    }
};
