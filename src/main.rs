mod detect;
mod ssd1306;

use linux_embedded_hal::I2cdev;

#[cfg(target_os = "linux")]
use linux_embedded_hal::Delay;

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

    if let Some((bus, address)) = detect::detect_display_i2c(9) {
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
            }
        }
    } else {
        eprintln!("Failed to automatically detect display, trying /dev/i2c-1");

        let default_address = detect::SSD1306_COMMON_ADDRESSES[0];

        match I2cdev::new("/dev/i2c-1") {
            Ok(i2c) => {
                eprintln!("I2C device /dev/i2c-1 successfully opened");
                let delay = Delay;
                ssd1306::init_oled(i2c, delay, default_address);
            }
            Err(e) => {
                eprintln!("ERROR: Failed to open I2C device /dev/i2c-1: {e:?}");
                eprintln!("Check I2C device path and access rights");

                eprintln!("I2C-1 bus scan results:");
                eprintln!("{}", detect::scan_i2c_bus(1));
            }
        }

    eprint!("Program execution completed");
    }
}
