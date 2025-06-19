mod detect;
mod eeprom;
mod ssd1306;

#[cfg(target_os = "linux")]
use linux_embedded_hal::Delay;

// Import I2cdev from detect module
use crate::detect::I2cdev;

#[cfg(not(target_os = "linux"))]
pub struct Delay;

#[cfg(not(target_os = "linux"))]
impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, _ns: u32) {}
    fn delay_us(&mut self, _us: u32) {}
    fn delay_ms(&mut self, _ms: u32) {}
}

fn main() {
    eprintln!("Ocultum: Initialization...");
    // 1. Получаем путь и адрес EEPROM HAT
    let (eeprom_path, eeprom_addr) = match eeprom::ddetect_eeprom_with_hat_id() {
        Some(pair) => pair,
        None => {
            eprintln!("EEPROM HAT не найден!");
            return;
        }
    };
    eprintln!("Найден EEPROM HAT: {eeprom_path} addr=0x{eeprom_addr:02X}");
    // 2. Читаем данные EEPROM
    let eeprom_data = match eeprom::read_eeprom_data(&eeprom_path, eeprom_addr) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Ошибка чтения EEPROM: {e:?}");
            return;
        }
    };
    // 3. Ищем дисплей
    let (bus, address) = match detect::detect_display(20) {
        Some(pair) => pair,
        None => {
            eprintln!("Дисплей не найден!");
            return;
        }
    };
    let i2c_path = format!("/dev/i2c-{bus}");
    let i2c = match I2cdev::new(&i2c_path) {
        Ok(i2c) => i2c,
        Err(e) => {
            eprintln!("Ошибка открытия I2C: {e:?}");
            return;
        }
    };
    let delay = Delay;
    // 4. Выводим данные EEPROM на дисплей
    ssd1306::display_eeprom_info(i2c, delay, address, &eeprom_data);
}
