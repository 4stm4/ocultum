sim800rs is a Rust library for working with SIM800 GSM/GPRS modules (UART, AT commands, SMS, GPRS, etc).

---

## Features
- UART (serial) communication with SIM800
- AT command interface
- SMS send/receive
- GPRS connect (APN)
- Configurable port, baudrate, PIN, APN
- No EEPROM required (settings via config)

## Example config (TOML)
```toml
uart_port = "/dev/serial0"
baudrate = 9600
sim_pin = "1234"
apn = "internet"
```

## Example usage
```rust
use sim800rs::{Sim800, Sim800Config};
let config = Sim800Config {
    uart_port: "/dev/serial0".to_string(),
    baudrate: 9600,
    sim_pin: Some("1234".to_string()),
    apn: Some("internet".to_string()),
};
let mut sim = Sim800::new(config).unwrap();
sim.send_sms("+79161234567", "Hello from Rust!").unwrap();
```

## Wiring
- Connect SIM800 TX/RX to Raspberry Pi UART (GPIO14/15 or USB-UART)
- Power: 3.3V/5V (see SIM800 datasheet)

## License
MIT
