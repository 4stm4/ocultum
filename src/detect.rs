// filepath: /Users/aleksejzaharcenko/work/ocultum/src/detect.rs
use embedded_hal::i2c::I2c;
use std::fmt::Write;
#[cfg(target_os = "linux")]
use std::fs;

#[cfg(target_os = "linux")]
pub use linux_embedded_hal::I2cdev;

#[cfg(not(target_os = "linux"))]
// Заглушка для I2cdev при компиляции на не-Linux платформах
pub struct I2cdev;

#[cfg(not(target_os = "linux"))]
#[derive(Debug)]
pub struct MockError;

#[cfg(not(target_os = "linux"))]
impl embedded_hal::i2c::Error for MockError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

#[cfg(not(target_os = "linux"))]
impl I2cdev {
    pub fn new(_path: &str) -> Result<Self, MockError> {
        Err(MockError)
    }
}

#[cfg(not(target_os = "linux"))]
impl embedded_hal::i2c::ErrorType for I2cdev {
    type Error = MockError;
}

#[cfg(not(target_os = "linux"))]
impl embedded_hal::i2c::I2c for I2cdev {
    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        Err(MockError)
    }
}

/// Константы адресов для распространенных I2C устройств
pub const SSD1306_COMMON_ADDRESSES: [u8; 2] = [0x3C, 0x3D];
/// Стандартный адрес HAT EEPROM на Raspberry Pi
pub const HAT_EEPROM_ADDRESS: u8 = 0x50;
/// Предопределенные типы устройств и их адреса для упрощения идентификации
pub const KNOWN_I2C_DEVICES: &[(&str, u8)] = &[
    ("SSD1306 Display", 0x3C),
    ("SSD1306 Display", 0x3D),
    ("HAT EEPROM", 0x50),
    ("BME280 Sensor", 0x76),
    ("BME280 Sensor", 0x77),
    ("MPU6050 Accel/Gyro", 0x68),
    ("MPU6050 Accel/Gyro", 0x69),
    ("PCA9685 PWM Controller", 0x40),
    ("ADS1115 ADC", 0x48),
    ("DS3231 RTC", 0x68),
];

/// Обнаруживает все доступные шины I2C в системе
///
/// На Linux-системах будет использована встроенная реализация для поиска устройств в /dev.
/// На других платформах возвращаются тестовые данные.
pub fn find_all_i2c_buses() -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        let mut buses = Vec::new();

        // Попытка найти все устройства I2C в /dev
        if let Ok(entries) = fs::read_dir("/dev") {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                #[allow(clippy::collapsible_if)]
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("i2c-") {
                        buses.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        buses.sort();
        buses
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Возвращаем имитацию шин для не-Linux систем
        let mut buses = Vec::new();
        buses.push("/dev/i2c-0".to_string());
        buses.push("/dev/i2c-1".to_string());
        buses
    }
}

pub fn detect_display_i2c(max_bus: u8) -> Option<(u8, u8)> {
    // Сначала пробуем использовать автоматическое обнаружение шин
    let i2c_buses = find_all_i2c_buses();

    // Пробуем обнаружить дисплей на всех найденных шинах
    for bus_path in &i2c_buses {
        if let Some(bus_num) = bus_path
            .strip_prefix("/dev/i2c-")
            .and_then(|num| num.parse::<u8>().ok())
        {
            match I2cdev::new(bus_path) {
                Ok(mut i2c) => {
                    for &addr in &SSD1306_COMMON_ADDRESSES {
                        let mut buf = [0u8; 1];
                        let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                        if i2c.transaction(addr, &mut ops).is_ok() {
                            println!("Display detected on bus {bus_num} at address 0x{addr:02X}");
                            return Some((bus_num, addr));
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }

    // Если автоматическое обнаружение не сработало, используем старый метод
    for bus in 0..=max_bus {
        let i2c_path = format!("/dev/i2c-{bus}");
        match I2cdev::new(&i2c_path) {
            Ok(mut i2c) => {
                for &addr in &SSD1306_COMMON_ADDRESSES {
                    let mut buf = [0u8; 1];
                    let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                    if i2c.transaction(addr, &mut ops).is_ok() {
                        println!("Display detected on bus {bus} at address 0x{addr:02X}");
                        return Some((bus, addr));
                    }
                }
            }
            Err(_) => {
                continue;
            }
        }
    }
    None
}

pub fn scan_i2c_bus(bus: u8) -> String {
    let mut result = String::new();
    let i2c_path = format!("/dev/i2c-{bus}");

    match I2cdev::new(&i2c_path) {
        Ok(mut i2c) => {
            writeln!(
                &mut result,
                "     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f"
            )
            .unwrap();

            for row in 0..8 {
                write!(&mut result, "{:02x}:", row * 16).unwrap();

                for col in 0..16 {
                    let addr = row * 16 + col;

                    if addr <= 7 || addr > 0x77 {
                        write!(&mut result, "   ").unwrap();
                        continue;
                    }

                    let mut buf = [0u8; 1];
                    let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                    let device_present = i2c.transaction(addr as u8, &mut ops).is_ok();

                    if device_present {
                        write!(&mut result, " {addr:02x}").unwrap();
                    } else {
                        write!(&mut result, " --").unwrap();
                    }
                }
                writeln!(&mut result).unwrap();
            }
        }
        Err(e) => {
            writeln!(&mut result, "Error opening I2C bus {i2c_path}: {e:?}").unwrap();
        }
    }

    result
}

/// Обнаруживает все доступные устройства на указанной шине I2C
///
/// Сканирует все возможные адреса I2C (от 0x08 до 0x77, исключая зарезервированные)
/// и возвращает список адресов устройств, которые ответили на запрос.
///
/// # Аргументы
///
/// * `bus_path` - Путь к шине I2C (например, "/dev/i2c-1")
///
/// # Возвращаемое значение
///
/// Вектор адресов (u8) найденных устройств на указанной шине.
pub fn find_devices_on_bus(bus_path: &str) -> Vec<u8> {
    let mut devices = Vec::new();

    match I2cdev::new(bus_path) {
        Ok(mut i2c) => {
            // Сканируем все возможные адреса I2C (от 0x08 до 0x77, исключая зарезервированные)
            for addr in 0x08..=0x77 {
                let mut buf = [0u8; 1];
                let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                if i2c.transaction(addr, &mut ops).is_ok() {
                    devices.push(addr);
                }
            }
        }
        Err(_) => {
            eprintln!("Failed to open I2C bus at {bus_path}");
        }
    }

    devices
}

/// Сканирует все шины I2C и возвращает список всех найденных устройств
///
/// Для каждой найденной шины I2C выполняет сканирование всех возможных
/// адресов и возвращает список пар (путь_к_шине, список_адресов_устройств).
///
/// # Возвращаемое значение
///
/// Вектор кортежей (String, Vec<u8>), где первый элемент - путь к шине I2C,
/// а второй - вектор адресов устройств, найденных на этой шине.
pub fn detect_all_i2c_devices() -> Vec<(String, Vec<u8>)> {
    let mut result = Vec::new();

    // Получаем список всех доступных шин I2C
    let buses = find_all_i2c_buses();

    for bus in buses {
        let devices = find_devices_on_bus(&bus);
        if !devices.is_empty() {
            result.push((bus, devices));
        }
    }

    result
}

/// Возвращает читаемое имя устройства по его адресу I2C, если оно известно
///
/// # Аргументы
///
/// * `addr` - Адрес I2C устройства
///
/// # Возвращаемое значение
///
/// Опциональная строка с названием устройства, если адрес соответствует известному устройству
pub fn get_device_name_by_address(addr: u8) -> Option<&'static str> {
    KNOWN_I2C_DEVICES
        .iter()
        .find(|(_, device_addr)| *device_addr == addr)
        .map(|(name, _)| *name)
}
