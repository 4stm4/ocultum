mod eeprom;
mod ssd1306;

use crate::ssd1306::display_eeprom_info;
use crate::ssd1306::set_eeprom;

fn main() {
    eprintln!("Ocultum: Initialization...");
    let (eeprom_path, eeprom_addr) = match eeprom::ddetect_eeprom_with_hat_id() {
        Some(pair) => pair,
        None => {
            eprintln!("EEPROM HAT не найден!");
            return;
        }
    };
    eprintln!("Найден EEPROM HAT: {eeprom_path} addr=0x{eeprom_addr:02X}");
    #[cfg(target_os = "linux")]
    {
        let eeprom_data = match eeprom::read_eeprom_data(&eeprom_path, eeprom_addr) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Ошибка чтения EEPROM: {e:?}");
                return;
            }
        };
        set_eeprom(eeprom_data);
        display_eeprom_info();
    }
}
