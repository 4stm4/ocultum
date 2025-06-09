//  _  _       _             _  _          
// | || |  ___| |_ _ __ ___ | || |         
// | || |_/ __| __| '_ ` _ \| || |_        
// |__   _\__ | |_| | | | | |__   _|       
//   |_| |___/\__|_|_|_| |_|  |_|         
//  ___   ___ _   _| | |_ _   _ _ __ ___  
// / _ \ / __| | | | | __| | | | '_ ` _ \ 
//| (_) | (__| |_| | | |_| |_| | | | | | |
// \___/ \___|\__,_|_|\__|\__,_|_| |_| |_|
//   sim800rs — Rust library for SIM800 GSM/GPRS modules
//
// Copyright (c) 2025 4stm4
// MIT License

//! # sim800rs
//!
//! Rust library for working with SIM800 GSM/GPRS modules (UART, AT commands, SMS, GPRS, etc).
//!
//! ## Example
//!
//! ```rust
//! use sim800rs::{Sim800, Sim800Config};
//!
//! let config = Sim800Config {
//!     uart_port: "/dev/serial0".to_string(),
//!     baudrate: 9600,
//!     sim_pin: Some("1234".to_string()),
//!     apn: Some("internet".to_string()),
//! };
//! let mut sim = Sim800::new(config).unwrap();
//! sim.send_sms("+79161234567", "Hello from Rust!").unwrap();
//! ```

/// SIM800 configuration
#[derive(Debug, Clone)]
pub struct Sim800Config {
    pub uart_port: String,
    pub baudrate: u32,
    pub sim_pin: Option<String>,
    pub apn: Option<String>,
}

/// Main SIM800 struct
pub struct Sim800 {
    config: Sim800Config,
    port: Box<dyn serialport::SerialPort>,
}

impl Sim800 {
    /// Create new SIM800 instance with config
    pub fn new(config: Sim800Config) -> Result<Self, &'static str> {
        let port = serialport::new(&config.uart_port, config.baudrate)
            .timeout(std::time::Duration::from_millis(1000))
            .open()
            .map_err(|_| "Failed to open serial port")?;
        // TODO: check connection, enter PIN, etc.
        Ok(Self { config, port })
    }

    /// Send raw AT command and get response
    pub fn send_at(&mut self, cmd: &str) -> Result<String, &'static str> {
        use std::io::{Read, Write};
        let at_cmd = format!("{}\r\n", cmd);
        self.port.write_all(at_cmd.as_bytes()).map_err(|_| "Write error")?;
        self.port.flush().map_err(|_| "Flush error")?;
        let mut buf = [0u8; 1024];
        let n = self.port.read(&mut buf).map_err(|_| "Read error")?;
        let resp = String::from_utf8_lossy(&buf[..n]).to_string();
        Ok(resp)
    }

    /// Send SMS to number
    pub fn send_sms(&mut self, _number: &str, _text: &str) -> Result<(), &'static str> {
        // TODO: AT+CMGS implementation
        Ok(())
    }

    /// Connect to GPRS (using APN)
    pub fn gprs_connect(&mut self) -> Result<(), &'static str> {
        // TODO: AT+CGATT, AT+CSTT, AT+CIICR, etc.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_create() {
        let config = Sim800Config {
            uart_port: "/dev/serial0".to_string(),
            baudrate: 9600,
            sim_pin: Some("1234".to_string()),
            apn: Some("internet".to_string()),
        };
        assert_eq!(config.baudrate, 9600);
    }
}
