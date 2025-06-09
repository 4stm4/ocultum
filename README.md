# ocultum

## Running tests

To run the library unit tests, execute:

```
cargo test
```

## Running real EEPROM read

To work with a real EEPROM via I2C (for example, on a Raspberry Pi):

1. Make sure the device is available at the required address (e.g., /dev/i2c-0, address 0x50).
2. Build and run the example:

```
cargo run
```

If device access is required, use:

```
sudo cargo run
```

## Code example

The file `src/main.rs` contains an example of reading EEPROM via I2C and checking data validity.
