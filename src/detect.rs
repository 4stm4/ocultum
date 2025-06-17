// filepath: /Users/aleksejzaharcenko/work/ocultum/src/detect.rs
use embedded_hal::i2c::I2c;
use std::fmt::Write;
#[cfg(target_os = "linux")]
use std::fs;

#[cfg(target_os = "linux")]
pub use linux_embedded_hal::I2cdev;

#[cfg(not(target_os = "linux"))]
// Mock implementation for I2cdev when compiling on non-Linux platforms
pub struct I2cdev;

#[cfg(not(target_os = "linux"))]
#[derive(Debug)]
pub struct MockError;

#[cfg(not(target_os = "linux"))]
impl embedded_hal::i2c::Error for MockError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

#[cfg(not(target_os = "linux"))]
impl I2cdev {
    pub fn new(_path: &str) -> Result<Self, MockError> {
        Err(MockError)
    }
}

#[cfg(not(target_os = "linux"))]
impl embedded_hal::i2c::ErrorType for I2cdev {
    type Error = MockError;
}

#[cfg(not(target_os = "linux"))]
impl embedded_hal::i2c::I2c for I2cdev {
    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Err(MockError)
    }
}

/// Constants for common I2C device addresses
pub const SSD1306_COMMON_ADDRESSES: [u8; 2] = [0x3C, 0x3D];
/// Standard address for HAT EEPROM on Raspberry Pi
#[allow(dead_code)]
pub const HAT_EEPROM_ADDRESS: u8 = 0x50;
/// Predefined device types and their addresses for easier identification
pub const KNOWN_I2C_DEVICES: &[(&str, u8)] = &[
    ("SSD1306 Display", 0x3C),
    ("SSD1306 Display", 0x3D),
    ("HAT EEPROM", 0x50),
    ("BME280 Sensor", 0x76),
    ("BME280 Sensor", 0x77),
    ("MPU6050 Accel/Gyro", 0x68),
    ("MPU6050 Accel/Gyro", 0x69),
    ("PCA9685 PWM Controller", 0x40),
    ("ADS1115 ADC", 0x48),
    ("DS3231 RTC", 0x68),
];

/// Detects all available I2C buses in the system
///
/// On Linux systems, the built-in implementation will be used to find devices in /dev.
/// On other platforms, test data is returned.
pub fn find_all_i2c_buses() -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        let mut buses = Vec::new();

        // Try to find all I2C devices in /dev
        if let Ok(entries) = fs::read_dir("/dev") {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                #[allow(clippy::collapsible_if)]
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("i2c-") {
                        buses.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        buses.sort();
        buses
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Return simulated buses for non-Linux systems
        let buses = vec!["/dev/i2c-0".to_string(), "/dev/i2c-1".to_string()];
        buses
    }
}

pub fn detect_display_i2c(max_bus: u8) -> Option<(u8, u8)> {
    // First try using automatic bus detection
    let i2c_buses = find_all_i2c_buses();

    // Try to detect display on all found buses
    for bus_path in &i2c_buses {
        if let Some(bus_num) = bus_path
            .strip_prefix("/dev/i2c-")
            .and_then(|num| num.parse::<u8>().ok())
        {
            match I2cdev::new(bus_path) {
                Ok(mut i2c) => {
                    for &addr in &SSD1306_COMMON_ADDRESSES {
                        let mut buf = [0u8; 1];
                        let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                        if i2c.transaction(addr, &mut ops).is_ok() {
                            println!("Display detected on bus {bus_num} at address 0x{addr:02X}");
                            return Some((bus_num, addr));
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }

    // If automatic detection didn't work, use the old method
    for bus in 0..=max_bus {
        let i2c_path = format!("/dev/i2c-{bus}");
        match I2cdev::new(&i2c_path) {
            Ok(mut i2c) => {
                for &addr in &SSD1306_COMMON_ADDRESSES {
                    let mut buf = [0u8; 1];
                    let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                    if i2c.transaction(addr, &mut ops).is_ok() {
                        println!("Display detected on bus {bus} at address 0x{addr:02X}");
                        return Some((bus, addr));
                    }
                }
            }
            Err(_) => {
                continue;
            }
        }
    }
    None
}

pub fn scan_i2c_bus(bus: u8) -> String {
    let mut result = String::new();
    let i2c_path = format!("/dev/i2c-{bus}");

    match I2cdev::new(&i2c_path) {
        Ok(mut i2c) => {
            writeln!(
                &mut result,
                "     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f"
            )
            .unwrap();

            for row in 0..8 {
                write!(&mut result, "{:02x}:", row * 16).unwrap();

                for col in 0..16 {
                    let addr = row * 16 + col;

                    if addr <= 7 || addr > 0x77 {
                        write!(&mut result, "   ").unwrap();
                        continue;
                    }

                    let mut buf = [0u8; 1];
                    let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                    let device_present = i2c.transaction(addr as u8, &mut ops).is_ok();

                    if device_present {
                        write!(&mut result, " {addr:02x}").unwrap();
                    } else {
                        write!(&mut result, " --").unwrap();
                    }
                }
                writeln!(&mut result).unwrap();
            }
        }
        Err(e) => {
            writeln!(&mut result, "Error opening I2C bus {i2c_path}: {e:?}").unwrap();
        }
    }

    result
}

/// Detects all available devices on the specified I2C bus
///
/// Scans all possible I2C addresses (from 0x08 to 0x77, excluding reserved ones)
/// and returns a list of device addresses that responded to the request.
///
/// # Arguments
///
/// * `bus_path` - Path to the I2C bus (e.g., "/dev/i2c-1")
///
/// # Returns
///
/// A vector of addresses (u8) of devices found on the specified bus.
pub fn find_devices_on_bus(bus_path: &str) -> Vec<u8> {
    let mut devices = Vec::new();

    match I2cdev::new(bus_path) {
        Ok(mut i2c) => {
            // Scan all possible I2C addresses (from 0x08 to 0x77, excluding reserved ones)
            for addr in 0x08..=0x77 {
                let mut buf = [0u8; 1];
                let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                if i2c.transaction(addr, &mut ops).is_ok() {
                    devices.push(addr);
                }
            }
        }
        Err(_) => {
            eprintln!("Failed to open I2C bus at {bus_path}");
        }
    }

    devices
}

/// Scans all I2C buses and returns a list of all found devices
///
/// For each discovered I2C bus, it scans all possible
/// addresses and returns a list of pairs (bus_path, list_of_device_addresses).
///
/// # Returns
///
/// A vector of tuples (String, `Vec<u8>`), where the first element is the path to the I2C bus,
/// and the second is a vector of device addresses found on that bus.
pub fn detect_all_i2c_devices() -> Vec<(String, Vec<u8>)> {
    let mut result = Vec::new();

    // Get a list of all available I2C buses
    let buses = find_all_i2c_buses();

    for bus in buses {
        let devices = find_devices_on_bus(&bus);
        if !devices.is_empty() {
            result.push((bus, devices));
        }
    }

    result
}

/// Returns the readable name of a device by its I2C address, if known
///
/// # Arguments
///
/// * `addr` - I2C device address
///
/// # Returns
///
/// Optional string with the device name, if the address corresponds to a known device
pub fn get_device_name_by_address(addr: u8) -> Option<&'static str> {
    KNOWN_I2C_DEVICES
        .iter()
        .find(|(_, device_addr)| *device_addr == addr)
        .map(|(name, _)| *name)
}
