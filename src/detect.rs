use embedded_hal::i2c::I2c;
use std::fmt::Write;

#[cfg(target_os = "linux")]
use linux_embedded_hal::I2cdev;

#[cfg(not(target_os = "linux"))]
// Заглушка для I2cdev при компиляции на не-Linux платформах
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

pub const SSD1306_COMMON_ADDRESSES: [u8; 2] = [0x3C, 0x3D];

pub fn detect_display_i2c(max_bus: u8) -> Option<(u8, u8)> {
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
