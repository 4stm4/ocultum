//! Core business logic

use crate::ports::{DisplayDetector, DisplayWriter, EepromReader};

pub struct OcultumService<E, D, DD>
where
    E: EepromReader,
    D: DisplayWriter,
    DD: DisplayDetector,
{
    eeprom_reader: E,
    display_writer: D,
    display_detector: DD,
}

impl<E, D, DD> OcultumService<E, D, DD>
where
    E: EepromReader,
    D: DisplayWriter,
    DD: DisplayDetector,
{
    pub fn new(eeprom_reader: E, display_writer: D, display_detector: DD) -> Self {
        Self {
            eeprom_reader,
            display_writer,
            display_detector,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Обнаружить EEPROM HAT
        let eeprom_device = self.eeprom_reader.detect_device()?;

        // 2. Прочитать информацию из EEPROM
        let device_info = self.eeprom_reader.read_device_info(&eeprom_device)?;

        // 3. Валидировать данные
        if !device_info.is_valid() {
            return Err("Invalid device info".into());
        }

        // 4. Обнаружить дисплей
        let display_device = self.display_detector.detect_display()?;

        // 5. Отобразить информацию на дисплее
        self.display_writer
            .display_info(&display_device, &device_info)?;

        Ok(())
    }
}
