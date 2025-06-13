#[cfg(feature = "linux")]
pub fn detect_and_show_eeprom_info(
    dev_path: &str,
    possible_addrs: &[u16],
    read_len: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::{Eeprom, read_from_eeprom_i2c};
    for &addr in possible_addrs {
        let mut buf = vec![0u8; read_len];
        if let Ok(_) = read_from_eeprom_i2c(&mut buf, dev_path, addr, 0) {
            if buf.len() >= 4 && &buf[0..4] == b"R-Pi" {
                match Eeprom::from_bytes(&buf) {
                    Ok(eeprom) => {
                        println!("EEPROM found at 0x{:02X} on {}", addr, dev_path);
                        println!("{eeprom}");
                        return Ok(());
                    }
                    Err(e) => {
                        println!("EEPROM found at 0x{:02X} but failed to parse: {}", addr, e);
                    }
                }
            }
        }
    }
    println!("No valid Raspberry Pi HAT EEPROM found on bus {dev_path}");
    Ok(())
}
