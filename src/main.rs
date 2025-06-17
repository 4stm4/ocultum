mod detect;
mod ssd1306;

#[cfg(target_os = "linux")]
use linux_embedded_hal::Delay;

// Import I2cdev from detect module
use crate::detect::I2cdev;

#[cfg(not(target_os = "linux"))]
pub struct Delay;

#[cfg(not(target_os = "linux"))]
impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, _ns: u32) {}
    fn delay_us(&mut self, _us: u32) {}
    fn delay_ms(&mut self, _ms: u32) {}
}

fn main() {
    eprintln!("Ocultum: Initialization...");
    eprintln!("Using ehatrom library for HAT data and I2C device detection");

    // Find all available I2C buses and devices on them
    let bus_devices = detect::detect_all_i2c_devices();

    if bus_devices.is_empty() {
        eprintln!("No I2C buses with devices detected!");
    } else {
        eprintln!("Found {} I2C buses with devices:", bus_devices.len());
        for (bus, devices) in &bus_devices {
            eprintln!("  Bus {}: {} devices: {:?}", bus, devices.len(), devices);
        }
    }

    // Try to detect display on all available buses
    if let Some((bus, address)) = detect::detect_display_i2c(20) {
        // Increased maximum bus number for search
        eprintln!("Automatically detected display on I2C-{bus} at address 0x{address:02X}");
        let i2c_path = format!("/dev/i2c-{bus}");
        match I2cdev::new(&i2c_path) {
            Ok(i2c) => {
                eprintln!("I2C device {i2c_path} successfully opened");
                let delay = Delay;
                ssd1306::init_oled(i2c, delay, address);
            }
            Err(e) => {
                eprintln!("ERROR: Failed to open I2C device {i2c_path}: {e:?}");
                try_fallback_display();
            }
        }
    } else {
        eprintln!("Failed to automatically detect display");
        try_fallback_display();
    }
}

// Function to attempt display connection on standard buses
fn try_fallback_display() {
    eprintln!("Trying fallback display connections...");

    // List of standard buses for Raspberry Pi
    let standard_buses = ["/dev/i2c-1", "/dev/i2c-0"];
    let default_address = detect::SSD1306_COMMON_ADDRESSES[0];

    for &bus_path in &standard_buses {
        eprintln!("Trying to connect to display on {bus_path}");
        match I2cdev::new(bus_path) {
            Ok(i2c) => {
                eprintln!("I2C device {bus_path} successfully opened");
                let delay = Delay;
                ssd1306::init_oled(i2c, delay, default_address);
                return; // Successfully connected to the display
            }
            Err(e) => {
                eprintln!("ERROR: Failed to open I2C device {bus_path}: {e:?}");
                continue; // Try the next bus
            }
        }
    }

    // If failed to connect to display, scan buses for debugging
    eprintln!("Failed to connect to display on standard buses");
    eprintln!("I2C bus scan results:");

    for i in 0..2 {
        eprintln!("I2C-{i} bus scan:");
        eprintln!("{}", detect::scan_i2c_bus(i));
    }

    eprintln!("Check I2C device paths and access rights");
    eprintln!("Program execution completed with errors");
}
