# ocultum

A minimal application for displaying Raspberry Pi HAT EEPROM information on an SSD1306 OLED display via I2C.

- Linux only (Raspberry Pi Zero 2W)
- Automatically detects the I2C display and reads EEPROM data
- Dependencies: ehatrom, ssd1306, embedded-graphics, embedded-hal, linux-embedded-hal

## Build and Run

```sh
make ci
sudo ./target/release/ocultum
```

## Docker (recommended)

```sh
./docker_build_run.sh
```

---

The project does not support other platforms.
