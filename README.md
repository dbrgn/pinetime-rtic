# RTFM on PineTime

Target MCU: nRF52832 (xxAA)

Current status: PoC

![img](demo.gif)

What works:

- Bare-metal Rust with [nrf52-hal](https://github.com/nrf-rs/nrf-hal)
- [RTFM](https://rtfm.rs/) for concurrency
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

To use `cargo run` below you need to have [rustup](https://rustup.rs/) installed. 

    $ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
To build for the nRF52832 install the correct ARM target.

    $ rustup target add thumbv7em-none-eabihf

`cargo run` uses `arm-none-eabi-gdb`. Use the [latest version from ARM's website](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads). From Ubuntu 18 onwards, the version obtained from `apt-get` is too old and does not work. Make sure to remove the old toolchain version.

    $ sudo apt remove binutils-arm-none-eabi gcc-arm-none-eabi libnewlib-arm-none-eabi

Untar the new package in your home directory (or wherever you like to have it installed):
    
    $ tar -xjvf gcc-arm-none-eabi-x-xxxx-qx-update-linux.tar.bz2

Add the new toolchain to your path:

    $ nano ~/.profile

At the bottom of the file add the following line (don't forget to replace xxxx with your version values)
```
export PATH=$PATH:/home/(your user)/gcc-arm-none-eabi-x-xxxx-qx-update/bin/
```

Reboot computer or run in terminal

    $ export PATH=$PATH:/home/(your user)/gcc-arm-none-eabi-x-xxxx-qx-update/bin/
    
Now you are done installing the ARM toolchain and you can use `cargo run`.


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

    $ cd pinetime-rtfm
    $ cargo run [--release]

gdb might notify you that auto-loading has been declined by some safety settings. To add an exception open your `.gdbinit` file

    $ nano ~/.gdbinit

At the bottom of that file add the rule (replace with the location of your own pinetime-rtfm directory location)
Hint: gdb will suggest what line to add to your `~/.gdbinit` file exactly in its warning message.

```
add-auto-load-safe-path /home/(user)/git_repos/pinetime-rtfm/pinetime-rtfm/.gdbinit
```
