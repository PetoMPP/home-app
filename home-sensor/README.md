# Home sensor

The ESP32 based sensor for the home app. Currently it is based on the ESP32-C3 board and is hosting an API for sending environmental data to the home api.
The temperature and humidity data is being developed.

# Schematics

The schematics for the sensor can be found in the `schematics` directory. The schematics are created using the KiCad EDA software.

# Building

To build the project, you need to have the Cargo build system installed. You can install it by following the instructions on the [Rust website](https://www.rust-lang.org/tools/install).

To make the build successful, you need to have the following environment variables set:
- `SSID` - the SSID of the WiFi network
- `WIFI_PASSWORD` - the password of the WiFi network

After installing Cargo, you can build the project by running the following command:

```bash
cargo build --release
```

The binary will be located in the `target/release` directory.

# Running

You can use the following command to flash the binary to the board:

```bash
cargo run --release
```

It will build the project and flash it to the board. The serial output will be in the terminal.
