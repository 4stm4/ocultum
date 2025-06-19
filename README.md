# ocultum

A minimal application for displaying Raspberry Pi HAT EEPROM information on an SSD1306 OLED display via I2C.

- Linux only (Raspberry Pi Zero 2W)  
- Clean hexagonal architecture (ports & adapters)
- Dependency injection and testable design
- Full SSD1306 display support with embedded-graphics
- Dependencies: ehatrom, ssd1306, embedded-graphics, embedded-hal, linux-embedded-hal

## Build and Run

```sh
cargo build --release
sudo ./target/release/ocultum
```

## Docker (recommended)

```sh
./docker_build_run.sh
```

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for details on the hexagonal architecture implementation.

---

The project does not support other platforms.
