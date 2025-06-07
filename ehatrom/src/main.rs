fn main() {
    // Пример: чтение EEPROM из файла или массива байт
    use ehatrom::Eeprom;
    use i2cdev::core::*;
    use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

    fn read_eeprom_i2c(dev_path: &str, addr: u16, len: usize) -> Result<Vec<u8>, LinuxI2CError> {
        let mut dev = LinuxI2CDevice::new(dev_path, addr)?;
        let mut buf = vec![0u8; len];
        // Для EEPROM HAT обычно читаем с 0 адреса
        dev.smbus_write_byte(0x00)?; // Установить адрес чтения в 0
        dev.read(&mut buf)?;
        Ok(buf)
    }

    // Путь к I2C устройству и адрес EEPROM
    let dev_path = "/dev/i2c-1";
    let addr = 0x50; // Стандартный адрес HAT EEPROM
    // Обычно EEPROM HAT 256 байт, но можно увеличить при необходимости
    let len = 256;
    let data = match read_eeprom_i2c(dev_path, addr, len) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Ошибка чтения с I2C: {}", e);
            return;
        }
    };
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
