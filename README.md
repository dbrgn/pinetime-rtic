# RTFM on PineTime

Target MCU: nRF52832 (xxAA)

Current status: PoC

## Flashing (cargo-flash)

Install cargo-flash:

    $ cargo install -f cargo-flash

Flash the target:

    $ cargo flash --chip nrf52832_xxAA

## Flashing (openocd)

Run OpenOCD:

    $ ./openocd.sh

Run the code

    $ cargo run [--release]

## Flashing (J-Link GDB Server)

Run JLinkGDBServer:

    $ ./jlinkgdbserver.sh

Run the code

    $ cargo run [--release]
