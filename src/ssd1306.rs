use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

pub fn init_oled<I2C, D>(i2c: I2C, _delay: D, address: u8)
where
    I2C: embedded_hal::i2c::I2c,
    I2C::Error: core::fmt::Debug,
    D: embedded_hal::delay::DelayNs,
{
    eprintln!("Initializing OLED display...");
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    if disp.init().is_err() {
        eprintln!("ERROR: Failed to initialize OLED display");
        return;
    }
    if disp.clear(BinaryColor::Off).is_err() {
        eprintln!("ERROR: Failed to clear OLED display");
        return;
    }

    display_ehatrom_info(&mut disp, address);
    
    // Update display content
    if let Err(e) = disp.flush() {
        eprintln!("ERROR: Failed to flush OLED display: {:?}", e);
        return;
    }
    
    eprintln!("OLED display initialized and running");
}

fn display_ehatrom_info<D>(disp: &mut D, _address: u8)
where
    D: DrawTarget<Color = BinaryColor>,
    D::Error: core::fmt::Debug,
{
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // Attempt to get ehatrom data
    let vendor_name = match get_ehatrom_vendor_info() {
        Some(name) => name,
        None => "Unknown vendor".to_string(),
    };
    
    let product_name = match get_ehatrom_product_name() {
        Some(name) => name,
        None => "Unknown product".to_string(),
    };
    
    let product_uuid = match get_ehatrom_product_uuid() {
        Some(uuid) => format!("UUID: {}", uuid),
        None => "No UUID found".to_string(),
    };

    if Text::with_baseline("Ocultum OLED", Point::new(0, 8), text_style, Baseline::Top)
        .draw(disp)
        .is_err()
        || Text::with_baseline(&vendor_name, Point::new(0, 22), text_style, Baseline::Top)
            .draw(disp)
            .is_err()
        || Text::with_baseline(
            &product_name,
            Point::new(0, 36),
            text_style,
            Baseline::Top,
        )
        .draw(disp)
        .is_err()
        || Text::with_baseline(&product_uuid, Point::new(0, 50), text_style, Baseline::Top)
            .draw(disp)
            .is_err()
    {
        eprintln!("ERROR: Failed to draw on OLED display");
        return;
    }
}

/// Gets vendor name from HAT EEPROM
fn get_ehatrom_vendor_info() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // Try to read EEPROM via ehatrom at standard HAT EEPROM address (0x50)
        let mut buffer = vec![0u8; 256]; // Buffer size for reading EEPROM
        
        match ehatrom::read_from_eeprom_i2c(&mut buffer, "/dev/i2c-0", 0x50, 0) {
            Ok(bytes_read) => {
                eprintln!("Successfully read {} bytes from HAT EEPROM", bytes_read);
                
                // Try to parse read data
                match ehatrom::Eeprom::from_bytes(&buffer[..bytes_read]) {
                    Ok(eeprom) => {
                        // Look for VendorInfo atom
                        for atom in eeprom.atoms() {
                            if let ehatrom::Atom::VendorInfo(vendor_info) = atom {
                                return Some(vendor_info.vendor_name().to_string());
                            }
                        }
                        None
                    }
                    Err(e) => {
                        eprintln!("Error parsing EEPROM data: {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from HAT EEPROM: {:?}", e);
                None
            }
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux platforms return test data
        Some("Simulated Vendor".to_string())
    }
}

/// Gets product name from HAT EEPROM
fn get_ehatrom_product_name() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // Try to read EEPROM via ehatrom at standard HAT EEPROM address (0x50)
        let mut buffer = vec![0u8; 256]; // Buffer size for reading EEPROM
        
        match ehatrom::read_from_eeprom_i2c(&mut buffer, "/dev/i2c-0", 0x50, 0) {
            Ok(bytes_read) => {
                // Try to parse read data
                match ehatrom::Eeprom::from_bytes(&buffer[..bytes_read]) {
                    Ok(eeprom) => {
                        // Look for VendorInfo atom
                        for atom in eeprom.atoms() {
                            if let ehatrom::Atom::VendorInfo(vendor_info) = atom {
                                return Some(vendor_info.product_name().to_string());
                            }
                        }
                        None
                    }
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux platforms return test data
        Some("Simulated Product".to_string())
    }
}

/// Получает UUID продукта из EEPROM HAT
fn get_ehatrom_product_uuid() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // Попытка прочитать EEPROM через ehatrom на стандартном адресе HAT EEPROM (0x50)
        let mut buffer = vec![0u8; 256]; // Размер буфера для чтения EEPROM
        
        match ehatrom::read_from_eeprom_i2c(&mut buffer, "/dev/i2c-0", 0x50, 0) {
            Ok(bytes_read) => {
                // Попытка распарсить прочитанные данные
                match ehatrom::Eeprom::from_bytes(&buffer[..bytes_read]) {
                    Ok(eeprom) => {
                        // Ищем атом с информацией о производителе
                        for atom in eeprom.atoms() {
                            if let ehatrom::Atom::VendorInfo(vendor_info) = atom {
                                // Форматируем байты UUID в строку
                                let uuid = vendor_info.product_uuid();
                                return Some(format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                                    uuid.time_low, uuid.time_mid, uuid.time_hi_version,
                                    (uuid.clock_seq_hi_variant as u16) << 8 | (uuid.clock_seq_low as u16),
                                    uuid.node));
                            }
                        }
                        None
                    }
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // На не-Linux платформах возвращаем тестовые данные
        Some("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string())
    }
}
