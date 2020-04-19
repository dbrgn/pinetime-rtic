# RTFM on PineTime

Target MCU: nRF52832 (xxAA)

Current status: PoC

## Flashing (cargo-embed)

Install cargo-embed:

    $ cargo install -f --git https://github.com/dbrgn/cargo-embed/ --branch config-improvements

Flash the target:

    $ cargo embed

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
