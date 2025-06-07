fn main() {
    // Пример: чтение EEPROM из файла или массива байт
    use std::fs::File;
    use std::io::Read;
    use ehatrom::Eeprom;

    // Замените путь на свой файл с дампом EEPROM
    let path = "eeprom_dump.bin";
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Не удалось открыть файл: {}", e);
            return;
        }
    };
    let mut data = Vec::new();
    if let Err(e) = file.read_to_end(&mut data) {
        eprintln!("Ошибка чтения файла: {}", e);
        return;
    }
    match Eeprom::from_bytes(&data) {
        Ok(eeprom) => {
            println!("EEPROM header: {:?}", eeprom.header);
            println!("Vendor info: {:?}", eeprom.vendor_info);
            println!("GPIO map bank0: {:?}", eeprom.gpio_map_bank0);
            if let Some(dt) = eeprom.dt_blob {
                println!("DT blob size: {} bytes", dt.len());
            }
            if let Some(gpio1) = eeprom.gpio_map_bank1 {
                println!("GPIO map bank1: {:?}", gpio1);
            }
        }
        Err(e) => {
            eprintln!("Ошибка парсинга EEPROM: {}", e);
        }
    }
}
