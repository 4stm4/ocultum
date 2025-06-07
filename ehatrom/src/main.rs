fn main() {
    // Пример: чтение EEPROM из файла или массива байт
    use ehatrom::{Eeprom, VendorInfoAtom, GpioMapAtom, write_to_eeprom_i2c, read_from_eeprom_i2c};
    use std::thread::sleep;
    use std::time::Duration;

    // --- Формируем структуру для записи ---
    let mut eeprom = Eeprom {
        header: ehatrom::EepromHeader {
            signature: *b"R-Pi",
            version: 1, // первая версия формата
            reserved: 0,
            numatoms: 0, // будет пересчитано
            eeplen: 0,   // будет пересчитано
        },
        vendor_info: VendorInfoAtom {
            vendor_id: 0x1234,
            product_id: 0x5678,
            product_ver: 1, // версия продукта в разработке — обычно 1
            vendor: {
                let mut arr = [0u8; 16];
                let s = b"4stm4";
                arr[..s.len()].copy_from_slice(s);
                arr
            },
            product: {
                let mut arr = [0u8; 16];
                let s = b"ocultum";
                arr[..s.len()].copy_from_slice(s);
                arr
            },
            uuid: [0u8; 16],
        },
        gpio_map_bank0: GpioMapAtom { flags: 0, pins: [0; 28] },
        dt_blob: None,
        gpio_map_bank1: None,
    };
    eeprom.update_header();
    // Можно добавить другие атомы через методы eeprom.add_*
    // --- Сериализация ---
    let eeprom_bytes = unsafe {
        // Примитивная сериализация: просто копируем память структур подряд
        // Для production лучше использовать byteorder или zerocopy
        let mut bytes = Vec::new();
        let header_ptr = &eeprom.header as *const _ as *const u8;
        bytes.extend_from_slice(std::slice::from_raw_parts(header_ptr, std::mem::size_of::<ehatrom::EepromHeader>()));
        // VendorInfo
        let atom_header = ehatrom::AtomHeader {
            atom_type: ehatrom::AtomType::VendorInfo as u8,
            count: 1,
            dlen: std::mem::size_of::<VendorInfoAtom>() as u16,
            reserved: 0,
        };
        let atom_ptr = &atom_header as *const _ as *const u8;
        bytes.extend_from_slice(std::slice::from_raw_parts(atom_ptr, std::mem::size_of::<ehatrom::AtomHeader>()));
        let vendor_ptr = &eeprom.vendor_info as *const _ as *const u8;
        bytes.extend_from_slice(std::slice::from_raw_parts(vendor_ptr, std::mem::size_of::<VendorInfoAtom>()));
        // GPIO bank0
        let atom_header = ehatrom::AtomHeader {
            atom_type: ehatrom::AtomType::GpioMapBank0 as u8,
            count: 1,
            dlen: std::mem::size_of::<GpioMapAtom>() as u16,
            reserved: 0,
        };
        let atom_ptr = &atom_header as *const _ as *const u8;
        bytes.extend_from_slice(std::slice::from_raw_parts(atom_ptr, std::mem::size_of::<ehatrom::AtomHeader>()));
        let gpio_ptr = &eeprom.gpio_map_bank0 as *const _ as *const u8;
        bytes.extend_from_slice(std::slice::from_raw_parts(gpio_ptr, std::mem::size_of::<GpioMapAtom>()));
        bytes
    };
    // --- Запись в EEPROM ---
    let dev_path = "/dev/i2c-0";
    let addr = 0x50;
    match write_to_eeprom_i2c(&eeprom_bytes, dev_path, addr) {
        Ok(_) => println!("Данные успешно записаны в EEPROM!"),
        Err(e) => {
            eprintln!("Ошибка записи в EEPROM: {}", e);
            return;
        }
    }
    // EEPROM может требовать задержку после записи
    sleep(Duration::from_millis(10));
    // --- Чтение и проверка ---
    let len = eeprom_bytes.len();
    let mut data = vec![0u8; len];
    match read_from_eeprom_i2c(&mut data, dev_path, addr, 0x0000) {
        Ok(_) => {
            // Для отладки: вывести первые 16 байт в hex
            print!("EEPROM HEX: ");
            for b in data.iter().take(16) {
                print!("{:02X} ", b);
            }
            println!("");
        },
        Err(e) => {
            eprintln!("Ошибка чтения с I2C: {}", e);
            return;
        }
    }
    match Eeprom::from_bytes(&data) {
        Ok(eeprom) => {
            if eeprom.is_valid() {
                println!("EEPROM header: {:?}", eeprom.header);
                println!("Vendor info: {:?}", eeprom.vendor_info);
                println!("GPIO map bank0: {:?}", eeprom.gpio_map_bank0);
            } else {
                println!("EEPROM пустой или неинициализированный (signature/version невалидны)");
            }
        }
        Err(e) => {
            eprintln!("Ошибка парсинга EEPROM: {}", e);
        }
    }
}
