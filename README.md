# Rust/RTIC on PineTime

Target MCU: nRF52832 (xxAA)

Current status: PoC

![img](demo.gif)

What works:

- Bare-metal Rust with [nrf52-hal](https://github.com/nrf-rs/nrf-hal)
- [RTIC](https://rtic.rs/) for concurrency
- [embedded-graphics](https://github.com/jamwaffles/embedded-graphics) for drawing onto the LCD
- Detect button presses
- Cycle through backlight brightness levels using button
- Show battery charge status and voltage
- Send BLE advertisement frames using the pure-Rust
  [rubble](https://github.com/jonas-schievink/rubble) stack

Planned:

- A simple watch interface
- Support for the step counter
- Better Bluetooth support
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
