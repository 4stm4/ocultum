use embedded_hal::i2c::I2c;
pub use linux_embedded_hal::I2cdev;

pub const SSD1306_COMMON_ADDRESSES: [u8; 2] = [0x3C, 0x3D];

#[allow(clippy::collapsible_if)]
fn find_all_i2c_buses() -> Vec<String> {
    let mut buses = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
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

pub fn detect_display(max_bus: u8) -> Option<(u8, u8)> {
    let i2c_buses = find_all_i2c_buses();
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
                            return Some((bus_num, addr));
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    }
    for bus in 0..=max_bus {
        let i2c_path = format!("/dev/i2c-{bus}");
        match I2cdev::new(&i2c_path) {
            Ok(mut i2c) => {
                for &addr in &SSD1306_COMMON_ADDRESSES {
                    let mut buf = [0u8; 1];
                    let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                    if i2c.transaction(addr, &mut ops).is_ok() {
                        return Some((bus, addr));
                    }
                }
            }
            Err(_) => continue,
        }
    }
    None
}
