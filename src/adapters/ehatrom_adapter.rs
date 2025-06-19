//! Адаптер для работы с EEPROM через ehatrom

use crate::core::domain::DeviceInfo;
use crate::ports::EepromReader;

#[cfg(target_os = "linux")]
use ehatrom::{Eeprom, EhatromError, find_i2c_devices, read_from_eeprom_i2c};

#[cfg(not(target_os = "linux"))]
use ehatrom::EhatromError;

pub struct EhatromAdapter;

#[derive(Debug)]
pub struct EepromDevice {
    pub path: String,
    pub address: u16,
}

impl EhatromAdapter {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "linux")]
    fn detect_eeprom_with_hat_id() -> Option<(String, u16)> {
        let devices = find_i2c_devices();
        let possible_addrs = [0x50u16];
        let read_len = std::env::var("EHATROM_BUFFER_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(32 * 1024);

        for dev in devices {
            for &addr in &possible_addrs {
                let mut buf = vec![0u8; read_len];
                if read_from_eeprom_i2c(&mut buf, &dev, addr, 0).is_ok()
                    && buf.len() >= 4
                    && &buf[0..4] == b"R-Pi"
                {
                    return Some((dev, addr));
                }
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    fn read_eeprom_data(dev_path: &str, addr: u16) -> Result<Eeprom, EhatromError> {
        let read_len = std::env::var("EHATROM_BUFFER_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(32 * 1024);
        let mut buf = vec![0u8; read_len];
        read_from_eeprom_i2c(&mut buf, dev_path, addr, 0)?;
        if buf.len() >= 4 && &buf[0..4] == b"R-Pi" {
            Eeprom::from_bytes(&buf).map_err(|_| EhatromError::InvalidData)
        } else {
            Err(EhatromError::InvalidData)
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn detect_eeprom_with_hat_id() -> Option<(String, u16)> {
        None
    }

    #[cfg(not(target_os = "linux"))]
    fn read_eeprom_data(_path: &str, _addr: u16) -> Result<ehatrom::Eeprom, EhatromError> {
        Err(EhatromError::InvalidData)
    }
}

impl EepromReader for EhatromAdapter {
    type EepromDevice = EepromDevice;
    type Error = EhatromError;

    fn detect_device(&self) -> Result<Self::EepromDevice, Self::Error> {
        Self::detect_eeprom_with_hat_id()
            .map(|(path, addr)| EepromDevice {
                path,
                address: addr,
            })
            .ok_or(EhatromError::InvalidData)
    }

    fn read_device_info(&self, device: &Self::EepromDevice) -> Result<DeviceInfo, Self::Error> {
        let eeprom = Self::read_eeprom_data(&device.path, device.address)?;
        let vendor = clean_string(&eeprom.vendor_info.vendor);
        let product = clean_string(&eeprom.vendor_info.product);
        Ok(DeviceInfo::new(vendor, product))
    }
}

fn clean_string(bytes: &[u8]) -> String {
    let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..null_pos])
        .unwrap_or("")
        .to_string()
}
