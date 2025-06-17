# ocultum

A project for working with the SSD1306 OLED display on Raspberry Pi Zero 2W using I2C interface. The application automatically detects the display on available I2C buses and outputs text to it.

> **Note**: This application is specifically designed to run on Raspberry Pi Zero 2W and is not compatible with other platforms.

## Subdirectories

- **ocultum** – main application for displaying text on the OLED screen.
- **sim800rs** – experimental library for interacting with SIM800 GSM/GPRS modules via UART.

## Dependencies

The project now uses the following external libraries:
- **ehatrom** (v0.3.1) - library for working with Raspberry Pi HAT EEPROM via I2C
- **ssd1306** - library for controlling SSD1306 OLED displays
- **embedded-graphics** - graphics library for embedded displays
- **embedded-hal** - hardware abstraction layer for embedded systems
- **linux-embedded-hal** - Linux implementation of embedded-hal

### Local Build on Raspberry Pi

```sh
# On the Raspberry Pi itself
cargo build --release
sudo ./target/release/ocultum
```

### Building and Running via Docker (recommended)

Docker provides an isolated environment and automatically installs all necessary dependencies.

```sh
# Running on Raspberry Pi via Docker
./docker_build_run.sh

# Local Docker run (only for testing on systems with I2C)
./docker_build_run.sh local
```

### Remote Build and Run (alternative method)

For building and running directly on Raspberry Pi:

```sh
./build_and_run.sh remote
```

or

```sh
./build_on_remote.sh run
```

> **Note**: SSH access to the Raspberry Pi must be configured. If necessary, modify the host and port in the build_on_remote.sh script.

## Functionality

- Automatic detection of OLED displays on I2C buses
- Scanning I2C buses for devices
- Displaying text on the OLED screen
- Reading and displaying Raspberry Pi HAT EEPROM data (vendor, product, and UUID information)
- Shows detailed HAT information using ehatrom library
- Auto-detection of HAT EEPROM on multiple I2C buses (/dev/i2c-0 and /dev/i2c-1)

## Requirements

- Raspberry Pi Zero 2W or other compatible single-board computer
- SSD1306 OLED display with I2C interface
- Connected display via I2C (typically SDA, SCL, GND, VCC pins)
- Enabled I2C interface on Raspberry Pi (`sudo raspi-config`)

## Raspberry Pi Setup

1. Enable I2C interface:
   ```sh
   sudo raspi-config
   # Select Interfacing Options -> I2C -> Yes
   ```

2. Install necessary packages:
   ```sh
   sudo apt update
   sudo apt install -y i2c-tools libi2c-dev
   ```

3. Check for I2C devices:
   ```sh
   sudo i2cdetect -y 1
   ```

## License

This project is distributed under the MIT license.
