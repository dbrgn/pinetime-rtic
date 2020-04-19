#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use embedded_graphics::prelude::*;
use embedded_graphics::{
    fonts::{Font12x16, Text},
    image::{Image, ImageRawLE},
    pixelcolor::Rgb565,
    primitives::rectangle::Rectangle,
    style::{PrimitiveStyleBuilder, TextStyleBuilder},
};
use nrf52832_hal::{self as hal, pac};
use nrf52832_hal::gpio::{Level, Output, PushPull, p0};
use nrf52832_hal::prelude::*;
use numtoa::NumToA;
use rtfm::app;
use rtfm::cyccnt::U32Ext;
use st7789::{self, Orientation};

mod delay;

static LCD_WIDTH: u16 = 240;
static LCD_HEIGHT: u16 = 240;

const CLOCK_FREQUENCY: u32 = 64_000_000;

#[app(device = nrf52832_hal::pac, peripherals = true, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        lcd: st7789::ST7789<
            hal::spim::Spim<pac::SPIM1>,
            p0::P0_18<Output<PushPull>>,
            p0::P0_26<Output<PushPull>>,
            delay::TimerDelay,
        >,
        text_style: TextStyleBuilder<Rgb565, Font12x16>,
        #[init(0)]
        counter: usize,
    }

    #[init(spawn = [write_counter])]
    fn init(cx: init::Context) -> init::LateResources {
        let _p = cx.core;
        let dp = cx.device;

        // Set up clocks
        let _clocks = hal::clocks::Clocks::new(dp.CLOCK);

        // Set up delay timer
        let delay = delay::TimerDelay::new(dp.TIMER0);

        // Set up GPIO peripheral
        let gpio = hal::gpio::p0::Parts::new(dp.P0);

        // Enable backlight
        let _backlight_low = gpio.p0_14.into_push_pull_output(Level::High);
        let _backlight_mid = gpio.p0_22.into_push_pull_output(Level::High);
        let mut backlight_high = gpio.p0_23.into_push_pull_output(Level::High);
        backlight_high.set_low().unwrap();

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

        // Chip select must be held low while driving the display. It must be high
        // when using other SPI devices on the same bus (such as external flash
        // storage) so that the display controller won't respond to the wrong
        // commands.
        lcd_cs.set_low().unwrap();

        // Initialize LCD
        let mut lcd = st7789::ST7789::new(spi, lcd_dc, lcd_rst, LCD_WIDTH, LCD_HEIGHT, delay);
        lcd.init().unwrap();
        lcd.set_orientation(&Orientation::Portrait).unwrap();

        // Draw something onto the LCD
        let background_color = Rgb565::new(0, 0b000111, 0);
        let backdrop_style = PrimitiveStyleBuilder::new()
            .fill_color(background_color)
            .build();
        let backdrop = Rectangle::new(
            Point::new(0, 0),
            Point::new(LCD_WIDTH as i32, LCD_HEIGHT as i32),
        ).into_styled(backdrop_style);
        backdrop.draw(&mut lcd).unwrap();
        let ferris_data = ImageRawLE::new(include_bytes!("../ferris.raw"), 86, 64);
        let ferris = Image::new(&ferris_data, Point::new(100, 80));
        ferris.draw(&mut lcd).unwrap();

        // Choose text style
        let text_style = TextStyleBuilder::new(Font12x16)
            .text_color(Rgb565::WHITE)
            .background_color(background_color);

        // Draw text
        Text::new("Hello world!", Point::new(10, 10))
            .into_styled(text_style.build())
            .draw(&mut lcd)
            .unwrap();

        // Schedule counter task immediately
        cx.spawn.write_counter().unwrap();

        init::LateResources {
            lcd,
            text_style,
        }
    }

    #[task(resources = [lcd, text_style, counter], schedule = [write_counter])]
    fn write_counter(cx: write_counter::Context) {
        // Write counter to the display
        let mut buf = [0u8; 20];
        let text = cx.resources.counter.numtoa_str(10, &mut buf);
        Text::new(text, Point::new(10, LCD_HEIGHT as i32 - 10 - 16))
            .into_styled(cx.resources.text_style.build())
            .draw(cx.resources.lcd)
            .unwrap();

        // Increment counter
        *cx.resources.counter += 1;

        // Re-schedule the timer interrupt
        cx.schedule
            .write_counter(cx.scheduled + CLOCK_FREQUENCY.cycles())
            .unwrap();
    }

    // Provide unused interrupts to RTFM for its scheduling
    extern "C" {
        fn SWI0_EGU0();
        fn SWI1_EGU1();
        fn SWI2_EGU2();
        fn SWI3_EGU3();
        fn SWI4_EGU4();
        fn SWI5_EGU5();
    }
};
