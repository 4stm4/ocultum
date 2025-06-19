use ehatrom::{Eeprom, EhatromError};
use ehatrom::{find_i2c_devices, read_from_eeprom_i2c};

pub fn ddetect_eeprom_with_hat_id() -> Option<(String, u16)> {
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

pub fn read_eeprom_data(dev_path: &str, addr: u16) -> Result<Eeprom, EhatromError> {
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
