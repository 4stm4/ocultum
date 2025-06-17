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

    // Сначала отображаем общую информацию о найденных I2C устройствах
    display_i2c_devices_info(&mut disp);

    // Ждем 5 секунд, чтобы пользователь мог прочитать информацию
    #[cfg(target_os = "linux")]
    {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // Затем отображаем данные HAT EEPROM
    display_ehatrom_info(&mut disp, address);

    // Обновляем содержимое дисплея
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

    // Отображаем заголовок
    if Text::with_baseline("Ocultum OLED", Point::new(0, 8), text_style, Baseline::Top)
        .draw(disp)
        .is_err()
    {
        eprintln!("ERROR: Failed to draw header on OLED display");
    }

    // Отображаем сообщение о сканировании
    if Text::with_baseline(
        "Scanning I2C buses...",
        Point::new(0, 18),
        text_style,
        Baseline::Top,
    )
    .draw(disp)
    .is_err()
    {
        eprintln!("ERROR: Failed to draw scanning message on OLED display");
    }

    // Attempt to get ehatrom data
    let ehatrom_data = get_ehatrom_data();

    // Очищаем дисплей перед отображением результатов
    if disp.clear(BinaryColor::Off).is_err() {
        eprintln!("ERROR: Failed to clear OLED display");
    }

    // Снова отображаем заголовок
    if Text::with_baseline("Ocultum OLED", Point::new(0, 8), text_style, Baseline::Top)
        .draw(disp)
        .is_err()
    {
        eprintln!("ERROR: Failed to draw header on OLED display");
    }

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

    // Добавляем информацию о шине I2C
    let bus_info = match ehatrom_data.bus_path {
        Some(bus) => format!("Bus: {bus}"),
        None => "No I2C bus found".to_string(),
    };

    // Добавляем информацию о других устройствах I2C
    let devices_info = match ehatrom_data.other_devices {
        Some(count) if count > 0 => format!("Found {count} I2C devices"),
        _ => "No other I2C devices".to_string(),
    };

    if Text::with_baseline(&vendor_name, Point::new(0, 18), text_style, Baseline::Top)
        .draw(disp)
        .is_err()
        || Text::with_baseline(&product_name, Point::new(0, 28), text_style, Baseline::Top)
            .draw(disp)
            .is_err()
        || Text::with_baseline(&bus_info, Point::new(0, 38), text_style, Baseline::Top)
            .draw(disp)
            .is_err()
        || Text::with_baseline(&devices_info, Point::new(0, 48), text_style, Baseline::Top)
            .draw(disp)
            .is_err()
        || Text::with_baseline(&product_uuid, Point::new(0, 58), text_style, Baseline::Top)
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
    bus_path: Option<String>,
    other_devices: Option<usize>, // Количество других обнаруженных устройств I2C
}

/// Читает все данные EEPROM и возвращает структуру с информацией
fn get_ehatrom_data() -> EhatromData {
    #[cfg(target_os = "linux")]
    {
        // Используем улучшенную функцию для обнаружения всех шин I2C и устройств на них
        let bus_devices = crate::detect::detect_all_i2c_devices();
        let eeprom_addr = crate::detect::HAT_EEPROM_ADDRESS;

        eprintln!("Detected {} I2C buses with devices:", bus_devices.len());

        let mut total_devices = 0;
        for (bus, devices) in &bus_devices {
            eprintln!("  Bus {}: {} devices: {:?}", bus, devices.len(), devices);
            total_devices += devices.len();
        }

        // Сначала ищем HAT EEPROM устройство на всех шинах
        for (bus, devices) in &bus_devices {
            if devices.contains(&eeprom_addr) {
                match read_ehatrom_from_bus(bus, eeprom_addr) {
                    Some(mut data) => {
                        // Добавляем информацию о других устройствах
                        data.other_devices = Some(total_devices);
                        return data;
                    }
                    None => continue,
                }
            }
        }

        // Если HAT EEPROM не обнаружен, но есть другие устройства, возвращаем информацию о них
        if !bus_devices.is_empty() {
            let (first_bus, _first_devices) = &bus_devices[0];
            return EhatromData {
                vendor_name: Some("No HAT detected".to_string()),
                product_name: Some(format!("{total_devices} devices found")),
                product_uuid: None,
                bus_path: Some(first_bus.clone()),
                other_devices: Some(total_devices),
            };
        }

        // Если не удалось прочитать с любой шины, возвращаем пустые данные
        EhatromData {
            vendor_name: None,
            product_name: None,
            product_uuid: None,
            bus_path: None,
            other_devices: None,
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // На не-Linux платформах возвращаем тестовые данные
        EhatromData {
            vendor_name: Some("Simulated Vendor".to_string()),
            product_name: Some("Simulated Product".to_string()),
            product_uuid: Some("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string()),
            bus_path: Some("/dev/i2c-0".to_string()),
            other_devices: Some(3), // Имитируем наличие 3 устройств
        }
    }
}

/// Читает данные EEPROM с указанной шины I2C с использованием функций ehatrom
#[cfg(target_os = "linux")]
fn read_ehatrom_from_bus(bus: &str, addr: u8) -> Option<EhatromData> {
    let mut buffer = vec![0u8; 256]; // Буфер для чтения EEPROM

    // Адрес должен быть u16 для этой функции
    let addr_u16: u16 = addr.into();

    match ehatrom::read_from_eeprom_i2c(&mut buffer, bus, addr_u16, 0) {
        Ok(_) => {
            // Определяем фактический размер данных (до первого нулевого байта или конца буфера)
            let bytes_read_usize = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
            eprintln!("Successfully read {bytes_read_usize} bytes from HAT EEPROM on {bus}");

            // Выводим первые 16 байт для диагностики
            if bytes_read_usize > 0 {
                let end_idx = std::cmp::min(16, bytes_read_usize);
                eprintln!("First 16 bytes: {:?}", &buffer[0..end_idx]);
            }

            // Проверяем сигнатуру Raspberry Pi HAT
            if bytes_read_usize >= 4 && &buffer[0..4] == b"R-Pi" {
                eprintln!("Found valid HAT signature on {bus}");
            } else if bytes_read_usize >= 4 {
                eprintln!(
                    "No HAT signature found on {bus} (first 4 bytes: {:?})",
                    &buffer[0..4]
                );
            }

            // Пробуем парсить прочитанные данные
            match ehatrom::Eeprom::from_bytes(&buffer) {
                Ok(eeprom) => {
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

                    eprintln!("Found HAT EEPROM data on {bus}:");
                    eprintln!("  Vendor: {vendor_str}");
                    eprintln!("  Product: {product_str}");
                    eprintln!("  UUID: {uuid_str}");

                    // Получаем количество устройств на этой шине
                    let all_buses = crate::detect::detect_all_i2c_devices();
                    let devices_count = all_buses.iter().map(|(_, devices)| devices.len()).sum();

                    return Some(EhatromData {
                        vendor_name: Some(if vendor_str.is_empty() {
                            "Unknown vendor".to_string()
                        } else {
                            vendor_str
                        }),
                        product_name: Some(if product_str.is_empty() {
                            "Unknown product".to_string()
                        } else {
                            product_str
                        }),
                        product_uuid: Some(uuid_str),
                        bus_path: Some(bus.to_string()), // Сохраняем путь к шине
                        other_devices: Some(devices_count),
                    });
                }
                Err(e) => {
                    eprintln!("Error parsing EEPROM data from {bus}: {e:?}");
                    // Если есть хотя бы какие-то данные, можно попытаться извлечь информацию
                    // из необработанных байтов EEPROM
                    if bytes_read_usize >= 16 {
                        eprintln!("Trying to extract basic info from raw bytes...");

                        // Проверяем, есть ли сигнатура Raspberry Pi
                        if &buffer[0..4] == b"R-Pi" {
                            let vendor_str =
                                format!("Unknown vendor (ID: {:02X}{:02X})", buffer[8], buffer[9]);
                            let product_str = format!("Unknown product (Bus: {bus})");

                            // Пытаемся сформировать UUID из доступных байтов
                            let uuid_str = if bytes_read_usize >= 32 {
                                format!(
                                    "Partial UUID: {:02x}{:02x}{:02x}{:02x}-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
                                    buffer[28], buffer[29], buffer[30], buffer[31]
                                )
                            } else {
                                "Unknown UUID".to_string()
                            };

                            // Получаем количество устройств на этой шине
                            let all_buses = crate::detect::detect_all_i2c_devices();
                            let devices_count =
                                all_buses.iter().map(|(_, devices)| devices.len()).sum();

                            return Some(EhatromData {
                                vendor_name: Some(vendor_str),
                                product_name: Some(product_str),
                                product_uuid: Some(uuid_str),
                                bus_path: Some(bus.to_string()),
                                other_devices: Some(devices_count),
                            });
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading from HAT EEPROM on {bus}: {e:?}");
        }
    }

    None
}

/// Отображает информацию о найденных I2C устройствах на OLED дисплее
pub fn display_i2c_devices_info<D>(disp: &mut D)
where
    D: DrawTarget<Color = BinaryColor>,
    D::Error: core::fmt::Debug,
{
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // Очищаем дисплей
    if disp.clear(BinaryColor::Off).is_err() {
        eprintln!("ERROR: Failed to clear OLED display");
        return;
    }

    // Отображаем заголовок
    if Text::with_baseline(
        "I2C Device Scanner",
        Point::new(0, 8),
        text_style,
        Baseline::Top,
    )
    .draw(disp)
    .is_err()
    {
        eprintln!("ERROR: Failed to draw header on OLED display");
    }

    // Получаем информацию о всех I2C устройствах
    let bus_devices = crate::detect::detect_all_i2c_devices();

    if bus_devices.is_empty() {
        // Если устройств не найдено
        if Text::with_baseline(
            "No I2C devices found",
            Point::new(0, 18),
            text_style,
            Baseline::Top,
        )
        .draw(disp)
        .is_err()
        {
            eprintln!("ERROR: Failed to draw on OLED display");
        }
    } else {
        // Отображаем информацию о найденных устройствах
        let mut y_pos = 18;
        let mut total_devices = 0;

        for (_idx, (bus, devices)) in bus_devices.iter().enumerate().take(3) {
            // Ограничиваем 3 шинами
            if y_pos > 50 {
                break; // Не выходим за пределы дисплея
            }

            let bus_name = if let Some(name) = bus.strip_prefix("/dev/") {
                name
            } else {
                bus
            };

            let line = format!("{}: {} devices", bus_name, devices.len());
            if Text::with_baseline(&line, Point::new(0, y_pos), text_style, Baseline::Top)
                .draw(disp)
                .is_err()
            {
                eprintln!("ERROR: Failed to draw on OLED display");
            }

            y_pos += 10;
            total_devices += devices.len();

            // Показываем до 2 устройств с их названиями для каждой шины
            for &addr in devices.iter().take(2) {
                if y_pos > 50 {
                    break; // Не выходим за пределы дисплея
                }

                let device_name = match crate::detect::get_device_name_by_address(addr) {
                    Some(name) => format!("{addr:02X}: {name}"),
                    None => format!("{addr:02X}: Unknown"),
                };

                if Text::with_baseline(
                    &device_name,
                    Point::new(5, y_pos),
                    text_style,
                    Baseline::Top,
                )
                .draw(disp)
                .is_err()
                {
                    eprintln!("ERROR: Failed to draw on OLED display");
                }

                y_pos += 10;
            }

            // Если на шине больше 2 устройств, показываем "... и еще N"
            if devices.len() > 2 {
                let more_text = format!("...and {} more", devices.len() - 2);
                if Text::with_baseline(&more_text, Point::new(5, y_pos), text_style, Baseline::Top)
                    .draw(disp)
                    .is_err()
                {
                    eprintln!("ERROR: Failed to draw on OLED display");
                }
                y_pos += 10;
            }
        }

        // Отображаем общее количество найденных устройств
        if (bus_devices.len() > 3 || y_pos > 50)
            && Text::with_baseline(
                &format!("Total: {total_devices} devices"),
                Point::new(0, 58),
                text_style,
                Baseline::Top,
            )
            .draw(disp)
            .is_err()
        {
            eprintln!("ERROR: Failed to draw on OLED display");
        }
    }
}
