//! Порты для взаимодействия с внешним миром

use crate::core::domain::{DeviceInfo, DisplayDevice};

/// Порт для чтения данных из EEPROM
pub trait EepromReader {
    type EepromDevice;
    type Error: std::error::Error + Send + Sync + 'static;

    fn detect_device(&self) -> Result<Self::EepromDevice, Self::Error>;
    fn read_device_info(&self, device: &Self::EepromDevice) -> Result<DeviceInfo, Self::Error>;
}

/// Порт для записи на дисплей
pub trait DisplayWriter {
    type Error: std::error::Error + Send + Sync + 'static;

    fn display_info(&self, device: &DisplayDevice, info: &DeviceInfo) -> Result<(), Self::Error>;
}

/// Порт для обнаружения дисплея
pub trait DisplayDetector {
    type Error: std::error::Error + Send + Sync + 'static;

    fn detect_display(&self) -> Result<DisplayDevice, Self::Error>;
}
