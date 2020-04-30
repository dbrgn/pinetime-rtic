# RTFM on PineTime

Target MCU: nRF52832 (xxAA)

Current status: PoC

![img](ferris.gif)

What works:

- Bare-metal Rust with [nrf52-hal](https://github.com/nrf-rs/nrf-hal)
- [RTFM](https://rtfm.rs/) for concurrency
- [embedded-graphics](https://github.com/jamwaffles/embedded-graphics) for drawing onto the LCD
- Detect button presses
- Cycle through backlight brightness levels using button

Planned:

- Battery level indicator
- A simple watch interface
- Support for the step counter
- Support for Bluetooth using [rubble](https://github.com/jonas-schievink/rubble),
  an experimental pure-rust BLE stack

## Development

### Flashing (cargo-embed)

Install cargo-embed:

    $ cargo install -f --git https://github.com/probe-rs/cargo-embed/

Flash the target:

    $ cargo embed --release

### Flashing (openocd)

Run OpenOCD:

    $ ./openocd.sh

Run the code

    $ cargo run [--release]

### Flashing (J-Link GDB Server)

Run JLinkGDBServer:

    $ ./jlinkgdbserver.sh

Run the code

    $ cargo run [--release]
