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
        eprintln!("ERROR: Failed to flush OLED display: {e:?}");
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
    let ehatrom_data = get_ehatrom_data();

    let vendor_name = ehatrom_data
        .vendor_name
        .unwrap_or_else(|| "Unknown vendor".to_string());
    let product_name = ehatrom_data
        .product_name
        .unwrap_or_else(|| "Unknown product".to_string());
    let product_uuid = match ehatrom_data.product_uuid {
        Some(uuid) => format!("UUID: {uuid}"),
        None => "No UUID found".to_string(),
    };

    if Text::with_baseline("Ocultum OLED", Point::new(0, 8), text_style, Baseline::Top)
        .draw(disp)
        .is_err()
        || Text::with_baseline(&vendor_name, Point::new(0, 22), text_style, Baseline::Top)
            .draw(disp)
            .is_err()
        || Text::with_baseline(&product_name, Point::new(0, 36), text_style, Baseline::Top)
            .draw(disp)
            .is_err()
        || Text::with_baseline(&product_uuid, Point::new(0, 50), text_style, Baseline::Top)
            .draw(disp)
            .is_err()
    {
        eprintln!("ERROR: Failed to draw on OLED display");
    }
}

/// Структура для хранения данных из HAT EEPROM
struct EhatromData {
    vendor_name: Option<String>,
    product_name: Option<String>,
    product_uuid: Option<String>,
}

/// Читает все данные EEPROM и возвращает структуру с информацией
fn get_ehatrom_data() -> EhatromData {
    #[cfg(target_os = "linux")]
    {
        // Список шин I2C для проверки
        let i2c_buses = ["/dev/i2c-0", "/dev/i2c-1"];
        let eeprom_addr = 0x50;

        for &bus in &i2c_buses {
            match read_ehatrom_from_bus(bus, eeprom_addr) {
                Some(data) => return data,
                None => continue,
            }
        }

        // Если не удалось прочитать с любой шины, возвращаем пустые данные
        EhatromData {
            vendor_name: None,
            product_name: None,
            product_uuid: None,
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // На не-Linux платформах возвращаем тестовые данные
        EhatromData {
            vendor_name: Some("Simulated Vendor".to_string()),
            product_name: Some("Simulated Product".to_string()),
            product_uuid: Some("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string()),
        }
    }
}

/// Читает данные EEPROM с указанной шины I2C
#[cfg(target_os = "linux")]
fn read_ehatrom_from_bus(bus: &str, addr: u8) -> Option<EhatromData> {
    let mut buffer = vec![0u8; 256]; // Буфер для чтения EEPROM

    // Адрес должен быть u16 для этой функции
    let addr_u16: u16 = addr.into();

    match ehatrom::read_from_eeprom_i2c(&mut buffer, bus, addr_u16, 0) {
        Ok(_) => {
            eprintln!("Successfully read bytes from HAT EEPROM on {bus}");

            // Пробуем парсить прочитанные данные
            if let Ok(eeprom) = ehatrom::Eeprom::from_bytes(&buffer) {
                // В версии 0.3.1 vendor_info - это поле, а не метод
                // Проверяем значение поля vendor_info
                let vendor_info = eeprom.vendor_info;
                // Преобразуем байтовые массивы в строки
                let vendor_str = String::from_utf8_lossy(
                    &vendor_info
                        .vendor
                        .iter()
                        .take_while(|&&b| b != 0)
                        .cloned()
                        .collect::<Vec<u8>>(),
                )
                .to_string();

                let product_str = String::from_utf8_lossy(
                    &vendor_info
                        .product
                        .iter()
                        .take_while(|&&b| b != 0)
                        .cloned()
                        .collect::<Vec<u8>>(),
                )
                .to_string();

                // Проверяем, есть ли данные
                if !vendor_str.is_empty() {
                    // Форматируем UUID (16 байт) в строку
                    let uuid_bytes = &vendor_info.uuid;
                    let uuid_str = format!(
                        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                        uuid_bytes[0],
                        uuid_bytes[1],
                        uuid_bytes[2],
                        uuid_bytes[3],
                        uuid_bytes[4],
                        uuid_bytes[5],
                        uuid_bytes[6],
                        uuid_bytes[7],
                        uuid_bytes[8],
                        uuid_bytes[9],
                        uuid_bytes[10],
                        uuid_bytes[11],
                        uuid_bytes[12],
                        uuid_bytes[13],
                        uuid_bytes[14],
                        uuid_bytes[15]
                    );

                    return Some(EhatromData {
                        vendor_name: Some(vendor_str),
                        product_name: Some(product_str),
                        product_uuid: Some(uuid_str),
                    });
                }
            } else {
                eprintln!("Error parsing EEPROM data from {bus}");
            }
        }
        Err(e) => {
            eprintln!("Error reading from HAT EEPROM on {bus}: {e:?}");
        }
    }

    None
}
