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

    // First display general information about detected I2C devices
    display_i2c_devices_info(&mut disp);

    // Wait 5 seconds so the user can read the information
    #[cfg(target_os = "linux")]
    {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // Then display HAT EEPROM data
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

    // Display header
    if Text::with_baseline("Ocultum OLED", Point::new(0, 8), text_style, Baseline::Top)
        .draw(disp)
        .is_err()
    {
        eprintln!("ERROR: Failed to draw header on OLED display");
    }

    // Display scanning message
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

    // Clear display before showing results
    if disp.clear(BinaryColor::Off).is_err() {
        eprintln!("ERROR: Failed to clear OLED display");
    }

    // Display header again
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

    // Add I2C bus information
    let bus_info = match ehatrom_data.bus_path {
        Some(bus) => format!("Bus: {bus}"),
        None => "No I2C bus found".to_string(),
    };

    // Add information about other I2C devices
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

/// Structure for storing HAT EEPROM data
struct EhatromData {
    vendor_name: Option<String>,
    product_name: Option<String>,
    product_uuid: Option<String>,
    bus_path: Option<String>,
    other_devices: Option<usize>, // Number of other detected I2C devices
}

/// Reads all EEPROM data and returns a structure with information
fn get_ehatrom_data() -> EhatromData {
    #[cfg(target_os = "linux")]
    {
        // Use improved function to detect all I2C buses and devices on them
        let bus_devices = crate::detect::detect_all_i2c_devices();
        let eeprom_addr = crate::detect::HAT_EEPROM_ADDRESS;

        eprintln!("Detected {} I2C buses with devices:", bus_devices.len());

        let mut total_devices = 0;
        for (bus, devices) in &bus_devices {
            eprintln!("  Bus {}: {} devices: {:?}", bus, devices.len(), devices);
            total_devices += devices.len();
        }

        // First, search for HAT EEPROM device on all buses
        for (bus, devices) in &bus_devices {
            if devices.contains(&eeprom_addr) {
                match read_ehatrom_from_bus(bus, eeprom_addr) {
                    Some(mut data) => {
                        // Add information about other devices
                        data.other_devices = Some(total_devices);
                        return data;
                    }
                    None => continue,
                }
            }
        }

        // If HAT EEPROM is not detected, but there are other devices, return information about them
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

        // If couldn't read from any bus, return empty data
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
        // On non-Linux platforms return test data
        EhatromData {
            vendor_name: Some("Simulated Vendor".to_string()),
            product_name: Some("Simulated Product".to_string()),
            product_uuid: Some("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string()),
            bus_path: Some("/dev/i2c-0".to_string()),
            other_devices: Some(3), // Simulate 3 devices
        }
    }
}

/// Reads EEPROM data from the specified I2C bus using ehatrom functions
#[cfg(target_os = "linux")]
fn read_ehatrom_from_bus(bus: &str, addr: u8) -> Option<EhatromData> {
    // Increase buffer to 512 bytes for more reliable reading
    let mut buffer = vec![0u8; 512];

    // Address should be u16 for this function
    let addr_u16: u16 = addr.into();

    // Try to read in blocks of different sizes for more reliable reading
    let mut total_bytes_read = 0;
    let mut read_error = false;

    // Try several different offsets and block sizes for reading
    for (offset, block_size) in [(0, 32), (0, 64), (0, 128), (0, 256), (0, 512)] {
        let mut temp_buffer = vec![0u8; block_size];
        match ehatrom::read_from_eeprom_i2c(&mut temp_buffer, bus, addr_u16, offset) {
            Ok(_) => {
                // Determine the number of bytes that appear to be meaningful data
                // HAT EEPROM may contain zero bytes in the middle of the data,
                // so we just accept at least 128 bytes or the entire buffer
                let read_bytes = if block_size <= 128 {
                    // For small blocks take everything we read
                    block_size
                } else {
                    // For large blocks, take either the block size or stop after 32 consecutive zero bytes
                    let mut last_non_zero = 0;
                    for (i, &byte) in temp_buffer.iter().enumerate() {
                        if byte != 0 {
                            last_non_zero = i;
                        } else if i > last_non_zero + 32 {
                            // Too many zeros in a row, probably end of data
                            break;
                        }
                    }
                    last_non_zero + 1
                };

                eprintln!(
                    "Read {read_bytes} bytes from offset {offset} (block size {block_size}) on {bus}"
                );

                // If we successfully read more data, use it
                if read_bytes > total_bytes_read {
                    total_bytes_read = read_bytes;
                    buffer[0..read_bytes].copy_from_slice(&temp_buffer[0..read_bytes]);
                    // If we read a full block, there might be more data
                    if read_bytes == block_size && block_size < 512 {
                        continue;
                    } else {
                        // We got an incomplete block or maximum size, no more data
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Error reading {block_size}-byte block at offset {offset} from HAT EEPROM on {bus}: {e:?}"
                );
                read_error = true;
                // Continue with other parameters
            }
        }
    }

    // If we couldn't read anything with any method
    if total_bytes_read == 0 {
        if read_error {
            eprintln!("Failed to read any data from HAT EEPROM on {bus} after multiple attempts");
        } else {
            eprintln!("No data received from HAT EEPROM on {bus}");
        }
        return None;
    }

    eprintln!("Successfully read {total_bytes_read} bytes from HAT EEPROM on {bus}");

    // Output more bytes for better diagnostics
    if total_bytes_read > 0 {
        // First 32 bytes
        let end_idx = std::cmp::min(32, total_bytes_read);
        let hex_bytes: Vec<String> = buffer[0..end_idx]
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect();
        eprintln!("First 32 bytes: [{}]", hex_bytes.join(", "));

        // Also output in ASCII format where possible
        let ascii_bytes: String = buffer[0..end_idx]
            .iter()
            .map(|&b| {
                if (32..=126).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        eprintln!("ASCII representation: [{ascii_bytes}]");
    }

    // Check different Raspberry Pi HAT signature formats
    let has_rpi_signature = if total_bytes_read >= 4 {
        // Standard "R-Pi" signature in ASCII
        let has_ascii_sig = &buffer[0..4] == b"R-Pi";

        // Hexadecimal representation of "R-Pi" [52, 2D, 50, 69]
        let has_hex_sig =
            buffer[0] == 0x52 && buffer[1] == 0x2D && buffer[2] == 0x50 && buffer[3] == 0x69;

        // Alternative formats that might be used
        let has_alt_sig1 =
            buffer[0] == b'R' && buffer[1] == b'-' && buffer[2] == b'P' && buffer[3] == b'i';

        has_ascii_sig || has_hex_sig || has_alt_sig1
    } else {
        false
    };

    if has_rpi_signature {
        eprintln!("Found valid HAT signature on {bus}");

        // Try to parse read data using ehatrom
        match ehatrom::Eeprom::from_bytes(&buffer[0..total_bytes_read]) {
            Ok(eeprom) => {
                // In version 0.3.1+, vendor_info is a field, not a method
                let vendor_info = eeprom.vendor_info;

                // Convert byte arrays to strings, handling null bytes
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

                // Format UUID (16 bytes) into a string
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

                eprintln!("Successfully parsed HAT EEPROM data on {bus}:");
                eprintln!("  Vendor: {vendor_str}");
                eprintln!("  Product: {product_str}");
                eprintln!("  UUID: {uuid_str}");

                // Get number of devices on this bus
                let all_buses = crate::detect::detect_all_i2c_devices();
                let devices_count = all_buses.iter().map(|(_, devices)| devices.len()).sum();

                // Output information about custom atoms for diagnostics
                let custom_atoms = eeprom.custom_atoms;
                if !custom_atoms.is_empty() {
                    eprintln!("  EEPROM contains {} custom atoms", custom_atoms.len());
                    for (i, atom) in custom_atoms.iter().enumerate() {
                        eprintln!(
                            "  Custom Atom {}: type={}, count={}",
                            i,
                            atom.0,
                            atom.1.len()
                        );
                    }
                } else {
                    eprintln!("  EEPROM contains no custom atoms");
                }

                Some(EhatromData {
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
                    bus_path: Some(bus.to_string()), // Save the bus path
                    other_devices: Some(devices_count),
                })
            }
            Err(e) => {
                eprintln!("Error parsing EEPROM data from {bus}: {e:?}");

                // Данные HAT EEPROM доступны, но их формат неправильный или неполный
                // Попробуем извлечь как можно больше информации из сырых данных

                // Вывести больше байтов в диагностических целях
                let diag_bytes = std::cmp::min(128, total_bytes_read);
                let hex_bytes: Vec<String> = buffer[0..diag_bytes]
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect();
                eprintln!(
                    "Raw HAT data (first {} bytes): [{}]",
                    diag_bytes,
                    hex_bytes.join(", ")
                );

                // Выводим также ASCII представление для лучшего анализа
                let ascii_bytes: String = buffer[0..diag_bytes]
                    .iter()
                    .map(|&b| {
                        if (32..=126).contains(&b) {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();
                eprintln!("ASCII representation: [{ascii_bytes}]");

                // Попытка проанализировать формат данных и структуру атома
                eprintln!("Analyzing HAT data format for error diagnostics:");
                if total_bytes_read >= 8 {
                    eprintln!(
                        "  Header bytes: [{:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}, {:02X}]",
                        buffer[0],
                        buffer[1],
                        buffer[2],
                        buffer[3],
                        buffer[4],
                        buffer[5],
                        buffer[6],
                        buffer[7]
                    );
                }

                // Проверка на наличие атомарной структуры
                // Обычно после заголовка "R-Pi" идут идентификаторы атомов
                let mut has_atom_structure = false;
                if total_bytes_read >= 12 {
                    // Проверяем байты 4-7 для идентификатора АТОМА и байты 8-11 для длины
                    eprintln!(
                        "  Possible atom header at offset 4: [{:02X}, {:02X}, {:02X}, {:02X}]",
                        buffer[4], buffer[5], buffer[6], buffer[7]
                    );
                    eprintln!(
                        "  Possible atom length at offset 8: [{:02X}, {:02X}, {:02X}, {:02X}]",
                        buffer[8], buffer[9], buffer[10], buffer[11]
                    );

                    // Простая эвристика для проверки формата атома
                    if buffer[4] < 16
                        && buffer[5] == 0
                        && buffer[6] == 0
                        && buffer[7] == 0
                        && buffer[8] < 128
                        && buffer[9] == 0
                        && buffer[10] == 0
                        && buffer[11] == 0
                    {
                        has_atom_structure = true;
                        eprintln!("  Data appears to have valid atom structure");
                    } else {
                        eprintln!("  Data does not appear to have standard atom structure");
                    }
                }

                // Попытка извлечь информацию о производителе и продукте из сырых данных
                // Согласно спецификации HAT:
                // - Смещение 8-9: Идентификатор производителя
                // - Позиции 16+ могут содержать данные атомов, включая имена
                let vendor_id = if total_bytes_read >= 10 {
                    format!("Vendor ID: {:02X}{:02X}", buffer[8], buffer[9])
                } else {
                    "Unknown Vendor".to_string()
                };

                // Попытаемся найти строки ASCII в данных для имен
                let mut product_name = "Unknown HAT Product".to_string();
                let mut vendor_name = vendor_id.clone();

                // Ищем последовательности ASCII в буфере, которые могут быть именами
                for i in 16..total_bytes_read.saturating_sub(4) {
                    // Проверяем только если текущий байт - печатаемый ASCII
                    if buffer[i] >= 32 && buffer[i] <= 126 {
                        let mut seq_len = 1;
                        // Проверяем следующие байты, пока не найдем не-ASCII или нулевой байт
                        while i + seq_len < total_bytes_read
                            && buffer[i + seq_len] >= 32
                            && buffer[i + seq_len] <= 126
                        {
                            seq_len += 1;
                        }

                        // Если нашли достаточно длинную последовательность, это может быть имя
                        if seq_len >= 4 {
                            let text = String::from_utf8_lossy(&buffer[i..i + seq_len]).to_string();
                            if product_name == "Unknown HAT Product" {
                                product_name = text;
                            } else if text.len() > product_name.len() {
                                // Если нашли более длинную строку, она может быть более информативной
                                vendor_name = product_name;
                                product_name = text;
                            }
                        }
                    }
                }

                // Попытка распарсить заголовок атома напрямую
                if has_atom_structure {
                    eprintln!("Attempting manual atom header parsing:");
                    // Атомы в HAT EEPROM следуют за 4-байтовой сигнатурой "R-Pi"
                    let mut offset = 4;
                    while offset + 8 <= total_bytes_read {
                        let atom_type = buffer[offset];
                        let atom_count = buffer[offset + 4];

                        eprintln!(
                            "  Atom at offset {offset}: type={atom_type}, count={atom_count}"
                        );

                        // Переходим к следующему атому, если есть
                        if atom_count == 0 {
                            break; // Недействительная длина атома
                        }
                        offset += 8 + atom_count as usize;
                    }
                }

                // Генерируем UUID из доступных байтов или используем смещение в буфере
                let uuid_str = if total_bytes_read >= 32 {
                    format!(
                        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                        buffer[16],
                        buffer[17],
                        buffer[18],
                        buffer[19],
                        buffer[20],
                        buffer[21],
                        buffer[22],
                        buffer[23],
                        buffer[24],
                        buffer[25],
                        buffer[26],
                        buffer[27],
                        buffer[28],
                        buffer[29],
                        buffer[30],
                        buffer[31]
                    )
                } else {
                    "00000000-0000-0000-0000-000000000000".to_string()
                };

                // Получаем количество устройств на этой шине
                let all_buses = crate::detect::detect_all_i2c_devices();
                let devices_count = all_buses.iter().map(|(_, devices)| devices.len()).sum();

                eprintln!("Fallback HAT parsing recovered the following:");
                eprintln!("  Vendor: {vendor_name}");
                eprintln!("  Product: {product_name}");
                eprintln!("  UUID: {uuid_str}");

                Some(EhatromData {
                    vendor_name: Some(vendor_name),
                    product_name: Some(product_name),
                    product_uuid: Some(uuid_str),
                    bus_path: Some(bus.to_string()),
                    other_devices: Some(devices_count),
                })
            }
        }
    } else {
        eprintln!(
            "No valid HAT signature found on {bus}, but device responded at address 0x{addr:02X}"
        );

        // Даже если нет правильной сигнатуры, устройство есть на шине I2C
        // Может быть другой формат или нестандартная EEPROM

        // Попытаемся найти строки ASCII в данных для имен
        let mut found_texts = Vec::new();

        // Ищем последовательности ASCII в буфере
        for i in 0..total_bytes_read.saturating_sub(4) {
            // Проверяем только если текущий байт - печатаемый ASCII
            if buffer[i] >= 32 && buffer[i] <= 126 {
                let mut seq_len = 1;
                // Проверяем следующие байты, пока не найдем не-ASCII или нулевой байт
                while i + seq_len < total_bytes_read
                    && buffer[i + seq_len] >= 32
                    && buffer[i + seq_len] <= 126
                {
                    seq_len += 1;
                }

                // Если нашли достаточно длинную последовательность, добавляем её
                if seq_len >= 4 {
                    let text = String::from_utf8_lossy(&buffer[i..i + seq_len]).to_string();
                    found_texts.push(text);
                }
            }
        }

        // Сортируем найденные строки по длине (длинные в начале)
        found_texts.sort_by_key(|b| std::cmp::Reverse(b.len()));

        // Выбираем лучшие кандидаты для имени продукта и производителя
        let product_name = if !found_texts.is_empty() {
            found_texts.remove(0)
        } else {
            "Unknown I2C EEPROM".to_string()
        };

        let vendor_name = if !found_texts.is_empty() {
            found_texts.remove(0)
        } else {
            "Unknown vendor".to_string()
        };

        // Получаем количество устройств на этой шине
        let all_buses = crate::detect::detect_all_i2c_devices();
        let devices_count = all_buses.iter().map(|(_, devices)| devices.len()).sum();

        eprintln!("Extracted text from non-HAT EEPROM:");
        eprintln!("  Vendor: {vendor_name}");
        eprintln!("  Product: {product_name}");

        Some(EhatromData {
            vendor_name: Some(vendor_name),
            product_name: Some(product_name),
            product_uuid: Some("00000000-0000-0000-0000-000000000000".to_string()),
            bus_path: Some(bus.to_string()),
            other_devices: Some(devices_count),
        })
    }
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
