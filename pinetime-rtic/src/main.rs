#![no_main]
#![cfg_attr(not(test), no_std)]

// Panic handler
#[cfg(not(test))]
use panic_rtt_target as _;

use debouncr::{debounce_6, Debouncer, Edge, Repeat6};
use embedded_graphics::prelude::*;
use embedded_graphics::{
    fonts::{Font12x16, Text},
    image::{Image, ImageRawLE},
    pixelcolor::Rgb565,
    primitives::rectangle::Rectangle,
    style::{PrimitiveStyleBuilder, TextStyleBuilder},
};
use nrf52832_hal::gpio::{p0, Floating, Input, Level, Output, Pin, PushPull};
use nrf52832_hal::prelude::*;
use nrf52832_hal::{self as hal, pac};
use numtoa::NumToA;
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use rubble::config::Config;
use rubble::gatt::BatteryServiceAttrs;
use rubble::l2cap::{BleChannelMap, L2CAPState};
use rubble::link::ad_structure::AdStructure;
use rubble::link::queue::{PacketQueue, SimpleQueue};
use rubble::link::{LinkLayer, Responder, MIN_PDU_BUF};
use rubble::security::NoSecurity;
use rubble::time::{Duration as RubbleDuration, Timer};
use rubble_nrf5x::radio::{BleRadio, PacketBuffer};
use rubble_nrf5x::timer::BleTimer;
use rubble_nrf5x::utils::get_device_address;
use st7789::{self, Orientation};

mod backlight;
mod battery;
mod delay;
mod monotonic_nrf52;

use monotonic_nrf52::U32Ext;

const LCD_W: u16 = 240;
const LCD_H: u16 = 240;

const FERRIS_W: u16 = 86;
const FERRIS_H: u16 = 64;

const MARGIN: u16 = 10;

const BACKGROUND_COLOR: Rgb565 = Rgb565::new(0, 0b000111, 0);

pub struct AppConfig {}

impl Config for AppConfig {
    type Timer = BleTimer<hal::target::TIMER2>;
    type Transmitter = BleRadio;
    type ChannelMapper = BleChannelMap<BatteryServiceAttrs, NoSecurity>;
    type PacketQueue = &'static mut SimpleQueue;
}

#[app(device = nrf52832_hal::pac, peripherals = true, monotonic = crate::monotonic_nrf52::Tim1)]
const APP: () = {
    struct Resources {
        // LCD
        lcd: st7789::ST7789<
            hal::spim::Spim<pac::SPIM1>,
            p0::P0_18<Output<PushPull>>,
            p0::P0_26<Output<PushPull>>,
            delay::TimerDelay,
        >,
        backlight: backlight::Backlight,

        // Battery
        battery: battery::BatteryStatus,

        // Button
        button: Pin<Input<Floating>>,
        button_debouncer: Debouncer<u8, Repeat6>,

        // Styles
        text_style: TextStyleBuilder<Rgb565, Font12x16>,

        // Counter resources
        #[init(0)]
        counter: usize,

        // Ferris resources
        ferris: ImageRawLE<'static, Rgb565>,
        #[init(10)]
        ferris_x_offset: i32,
        #[init(80)]
        ferris_y_offset: i32,
        #[init(2)]
        ferris_step_size: i32,

        // BLE
        #[init([0; MIN_PDU_BUF])]
        ble_tx_buf: PacketBuffer,
        #[init([0; MIN_PDU_BUF])]
        ble_rx_buf: PacketBuffer,
        #[init(SimpleQueue::new())]
        tx_queue: SimpleQueue,
        #[init(SimpleQueue::new())]
        rx_queue: SimpleQueue,
        radio: BleRadio,
        ble_ll: LinkLayer<AppConfig>,
        ble_r: Responder<AppConfig>,
    }

    #[init(
        resources = [ble_tx_buf, ble_rx_buf, tx_queue, rx_queue],
        spawn = [write_counter, write_ferris, poll_button, show_battery_status, update_battery_status],
    )]
    fn init(cx: init::Context) -> init::LateResources {
        // Destructure device peripherals
        let pac::Peripherals {
            CLOCK,
            FICR,
            P0,
            RADIO,
            SAADC,
            SPIM1,
            TIMER0,
            TIMER1,
            TIMER2,
            ..
        } = cx.device;

        // Init RTT
        rtt_init_print!();
        rprintln!("Initializing…");

        // Set up clocks. On reset, the high frequency clock is already used,
        // but we also need to switch to the external HF oscillator. This is
        // needed for Bluetooth to work.
        let _clocks = hal::clocks::Clocks::new(CLOCK).enable_ext_hfosc();

        // Set up delay provider on TIMER0
        let delay = delay::TimerDelay::new(TIMER0);

        // Initialize monotonic timer on TIMER1 (for RTIC)
        monotonic_nrf52::Tim1::initialize(TIMER1);

        // Initialize BLE timer on TIMER2
        let ble_timer = BleTimer::init(TIMER2);

        // Set up GPIO peripheral
        let gpio = hal::gpio::p0::Parts::new(P0);

        // Enable backlight
        let backlight = backlight::Backlight::init(
            gpio.p0_14.into_push_pull_output(Level::High).degrade(),
            gpio.p0_22.into_push_pull_output(Level::High).degrade(),
            gpio.p0_23.into_push_pull_output(Level::High).degrade(),
            1,
        );

        // Battery status
        let battery = battery::BatteryStatus::init(
            gpio.p0_12.into_floating_input(),
            gpio.p0_31.into_floating_input(),
            SAADC,
        );

        // Enable button
        gpio.p0_15.into_push_pull_output(Level::High);
        let button = gpio.p0_13.into_floating_input().degrade();

        // Get bluetooth device address
        let device_address = get_device_address();
        rprintln!("Bluetooth device address: {:?}", device_address);

        // Initialize radio
        let mut radio = BleRadio::new(
            RADIO,
            &FICR,
            cx.resources.ble_tx_buf,
            cx.resources.ble_rx_buf,
        );

        // Create bluetooth TX/RX queues
        let (tx, tx_cons) = cx.resources.tx_queue.split();
        let (rx_prod, rx) = cx.resources.rx_queue.split();

        // Create the actual BLE stack objects
        let mut ble_ll = LinkLayer::<AppConfig>::new(device_address, ble_timer);
        let ble_r = Responder::<AppConfig>::new(
            tx,
            rx,
            L2CAPState::new(BleChannelMap::with_attributes(BatteryServiceAttrs::new())),
        );

        // Send advertisement and set up regular interrupt
        let next_update = ble_ll
            .start_advertise(
                RubbleDuration::from_millis(200),
                &[AdStructure::CompleteLocalName("Rusty PineTime")],
                &mut radio,
                tx_cons,
                rx_prod,
            )
            .unwrap();
        ble_ll.timer().configure_interrupt(next_update);

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
            SPIM1,
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
        let mut lcd = st7789::ST7789::new(spi, lcd_dc, lcd_rst, LCD_W, LCD_H, delay);
        lcd.init().unwrap();
        lcd.set_orientation(&Orientation::Portrait).unwrap();

        // Draw something onto the LCD
        let backdrop_style = PrimitiveStyleBuilder::new()
            .fill_color(BACKGROUND_COLOR)
            .build();
        Rectangle::new(Point::new(0, 0), Point::new(LCD_W as i32, LCD_H as i32))
            .into_styled(backdrop_style)
            .draw(&mut lcd)
            .unwrap();

        // Choose text style
        let text_style = TextStyleBuilder::new(Font12x16)
            .text_color(Rgb565::WHITE)
            .background_color(BACKGROUND_COLOR);

        // Draw text
        Text::new("PineTime", Point::new(10, 10))
            .into_styled(text_style.build())
            .draw(&mut lcd)
            .unwrap();

        // Load ferris image data
        let ferris = ImageRawLE::new(
            include_bytes!("../ferris.raw"),
            FERRIS_W as u32,
            FERRIS_H as u32,
        );

        // Schedule tasks immediately
        cx.spawn.write_counter().unwrap();
        cx.spawn.write_ferris().unwrap();
        cx.spawn.poll_button().unwrap();
        cx.spawn.show_battery_status().unwrap();
        cx.spawn.update_battery_status().unwrap();

        init::LateResources {
            lcd,
            battery,
            backlight,
            button,
            button_debouncer: debounce_6(),
            text_style,
            ferris,

            radio,
            ble_ll,
            ble_r,
        }
    }

    /// Hook up the RADIO interrupt to the Rubble BLE stack.
    #[task(binds = RADIO, resources = [radio, ble_ll], spawn = [ble_worker], priority = 3)]
    fn radio(cx: radio::Context) {
        let ble_ll: &mut LinkLayer<AppConfig> = cx.resources.ble_ll;
        if let Some(cmd) = cx
            .resources
            .radio
            .recv_interrupt(ble_ll.timer().now(), ble_ll)
        {
            cx.resources.radio.configure_receiver(cmd.radio);
            ble_ll.timer().configure_interrupt(cmd.next_update);

            if cmd.queued_work {
                // If there's any lower-priority work to be done, ensure that happens.
                // If we fail to spawn the task, it's already scheduled.
                cx.spawn.ble_worker().ok();
            }
        }
    }

    /// Hook up the TIMER2 interrupt to the Rubble BLE stack.
    #[task(binds = TIMER2, resources = [radio, ble_ll], spawn = [ble_worker], priority = 3)]
    fn timer2(cx: timer2::Context) {
        let timer = cx.resources.ble_ll.timer();
        if !timer.is_interrupt_pending() {
            return;
        }
        timer.clear_interrupt();

        let cmd = cx.resources.ble_ll.update_timer(&mut *cx.resources.radio);
        cx.resources.radio.configure_receiver(cmd.radio);

        cx.resources
            .ble_ll
            .timer()
            .configure_interrupt(cmd.next_update);

        if cmd.queued_work {
            // If there's any lower-priority work to be done, ensure that happens.
            // If we fail to spawn the task, it's already scheduled.
            cx.spawn.ble_worker().ok();
        }
    }

    /// Lower-priority task spawned from RADIO and TIMER2 interrupts.
    #[task(resources = [ble_r], priority = 2)]
    fn ble_worker(cx: ble_worker::Context) {
        // Fully drain the packet queue
        while cx.resources.ble_r.has_work() {
            cx.resources.ble_r.process_one().unwrap();
        }
    }

    #[task(resources = [lcd, ferris, ferris_x_offset, ferris_y_offset, ferris_step_size], schedule = [write_ferris])]
    fn write_ferris(cx: write_ferris::Context) {
        // Draw ferris
        Image::new(
            &cx.resources.ferris,
            Point::new(*cx.resources.ferris_x_offset, *cx.resources.ferris_y_offset),
        )
        .draw(cx.resources.lcd)
        .unwrap();

        // Clean up behind ferris
        let backdrop_style = PrimitiveStyleBuilder::new()
            .fill_color(BACKGROUND_COLOR)
            .build();
        let (p1, p2) = if *cx.resources.ferris_step_size > 0 {
            // Clean up to the left
            (
                Point::new(
                    *cx.resources.ferris_x_offset - *cx.resources.ferris_step_size,
                    *cx.resources.ferris_y_offset,
                ),
                Point::new(
                    *cx.resources.ferris_x_offset,
                    *cx.resources.ferris_y_offset + (FERRIS_H as i32),
                ),
            )
        } else {
            // Clean up to the right
            (
                Point::new(
                    *cx.resources.ferris_x_offset + FERRIS_W as i32,
                    *cx.resources.ferris_y_offset,
                ),
                Point::new(
                    *cx.resources.ferris_x_offset + FERRIS_W as i32
                        - *cx.resources.ferris_step_size,
                    *cx.resources.ferris_y_offset + (FERRIS_H as i32),
                ),
            )
        };
        Rectangle::new(p1, p2)
            .into_styled(backdrop_style)
            .draw(cx.resources.lcd)
            .unwrap();

        // Reset step size
        if *cx.resources.ferris_x_offset as u16 > LCD_W - FERRIS_W - MARGIN {
            *cx.resources.ferris_step_size = -*cx.resources.ferris_step_size;
        } else if (*cx.resources.ferris_x_offset as u16) < MARGIN {
            *cx.resources.ferris_step_size = -*cx.resources.ferris_step_size;
        }
        *cx.resources.ferris_x_offset += *cx.resources.ferris_step_size;

        // Re-schedule the timer interrupt
        cx.schedule.write_ferris(cx.scheduled + 25.hz()).unwrap();
    }

    #[task(resources = [lcd, text_style, counter], schedule = [write_counter])]
    fn write_counter(cx: write_counter::Context) {
        rprintln!("Counter is {}", cx.resources.counter);

        // Write counter to the display
        let mut buf = [0u8; 20];
        let text = cx.resources.counter.numtoa_str(10, &mut buf);
        Text::new(text, Point::new(10, LCD_H as i32 - 10 - 16))
            .into_styled(cx.resources.text_style.build())
            .draw(cx.resources.lcd)
            .unwrap();

        // Increment counter
        *cx.resources.counter += 1;

        // Re-schedule the timer interrupt
        cx.schedule.write_counter(cx.scheduled + 1.secs()).unwrap();
    }

    #[task(resources = [button, button_debouncer], spawn = [button_pressed], schedule = [poll_button])]
    fn poll_button(cx: poll_button::Context) {
        // Poll button
        let pressed = cx.resources.button.is_high().unwrap();
        let edge = cx.resources.button_debouncer.update(pressed);

        // Dispatch event
        if edge == Some(Edge::Rising) {
            cx.spawn.button_pressed().unwrap();
        }

        // Re-schedule the timer interrupt in 2ms
        cx.schedule.poll_button(cx.scheduled + 2.millis()).unwrap();
    }

    /// Called when button is pressed without bouncing for 12 (6 * 2) ms.
    #[task(resources = [backlight])]
    fn button_pressed(cx: button_pressed::Context) {
        if cx.resources.backlight.get_brightness() < 7 {
            cx.resources.backlight.brighter();
        } else {
            cx.resources.backlight.off();
        }
    }

    /// Fetch the battery status from the hardware. Update the text if
    /// something changed.
    #[task(resources = [battery], spawn = [show_battery_status], schedule = [update_battery_status])]
    fn update_battery_status(cx: update_battery_status::Context) {
        rprintln!("Update battery status");

        let changed = cx.resources.battery.update();
        if changed {
            rprintln!("Battery status changed");
            cx.spawn.show_battery_status().unwrap();
        }

        // Re-schedule the timer interrupt in 1s
        cx.schedule
            .update_battery_status(cx.scheduled + 1.secs())
            .unwrap();
    }

    /// Show the battery status on the LCD.
    #[task(resources = [battery, lcd, text_style])]
    fn show_battery_status(cx: show_battery_status::Context) {
        let voltage = cx.resources.battery.voltage();
        let charging = cx.resources.battery.is_charging();

        rprintln!(
            "Battery status: {} ({})",
            voltage,
            if charging { "charging" } else { "discharging" },
        );

        // Show battery status in top right corner
        let mut buf = [0u8; 6];
        (voltage / 10).numtoa(10, &mut buf[0..1]);
        buf[1] = b'.';
        (voltage % 10).numtoa(10, &mut buf[2..3]);
        buf[3] = b'V';
        buf[4] = b'/';
        buf[5] = if charging { b'C' } else { b'D' };
        let status = core::str::from_utf8(&buf).unwrap();
        let text = Text::new(status, Point::zero()).into_styled(cx.resources.text_style.build());
        let translation = Point::new(
            LCD_W as i32 - text.size().width as i32 - MARGIN as i32,
            MARGIN as i32,
        );
        text.translate(translation).draw(cx.resources.lcd).unwrap();
    }

    // Provide unused interrupts to RTIC for its scheduling
    extern "C" {
        fn SWI0_EGU0();
        fn SWI1_EGU1();
        fn SWI2_EGU2();
        fn SWI3_EGU3();
        fn SWI4_EGU4();
        fn SWI5_EGU5();
    }
};
