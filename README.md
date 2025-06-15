# ocultum

This repository groups a few Rust crates targeting small embedded setups such as Raspberry Pi or similar boards. The top level crate `ocultum` demonstrates basic usage of an SSD1306 OLED display using `embedded-graphics`.

## Subcrates

- **ocultum** – example application showing text on an OLED screen. [Documentation](https://docs.rs/?crate=ocultum "Generated docs") will be available once published with `cargo doc`.
- **ehatrom** – library for creating and handling Raspberry Pi HAT EEPROM images. [Documentation](https://docs.rs/ehatrom) is published on docs.rs.
- **sim800rs** – experimental library for interacting with SIM800 GSM/GPRS modules over UART. [Documentation](https://docs.rs/?crate=sim800rs "Generated docs") can be generated locally.

## Building

Ensure you have a recent Rust toolchain (nightly is required for CI). To build all crates in the workspace:

```sh
cargo build --workspace
```

### Cross compilation

For cross‑compiling to `armv7-unknown-linux-gnueabihf`, use:

```sh
./cross_compile.sh
```

### Uploading and running

For quick deployment to a Raspberry Pi after building, use:

```sh
./upload_and_run.sh
```

The `ehatrom` crate also provides `ehatrom/update_and_run.sh` which demonstrates reading and writing real HAT EEPROMs.

## License

This project is licensed under the MIT license.
