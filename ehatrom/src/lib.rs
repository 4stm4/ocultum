use core::fmt;
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EepromHeader {
    pub signature: [u8; 4],  // Always 0x52 0x2D 0x50 0x69 ("R-Pi")
    pub version: u8,         // Format version (0x01 for first version)
    pub reserved: u8,        // 0x00
    pub numatoms: u16,       // Количество атомов (Little Endian)
    pub eeplen: u32,         // Общая длина данных EEPROM (LE)
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct AtomHeader {
    pub atom_type: u8,    // Тип атома
    pub count: u8,        // Количество структур в атоме (обычно 1)
    pub dlen: u16,        // Длина данных атома (LE)
    pub reserved: u32,    // Зарезервировано (0)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomType {
    VendorInfo = 0x01,
    GpioMapBank0 = 0x02,
    DtBlob = 0x03,
    GpioMapBank1 = 0x04,
    Unknown,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct VendorInfoAtom {
    pub vendor_id: u16,      // ID производителя
    pub product_id: u16,     // ID продукта
    pub product_ver: u16,    // Версия продукта
    pub vendor: [u8; 16],    // Имя производителя (null-terminated)
    pub product: [u8; 16],   // Имя продукта (null-terminated)
    pub uuid: [u8; 16],      // UUID
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GpioMapAtom {
    pub flags: u16,          // Флаги GPIO
    pub pins: [u8; 28],      // Карта пинов (28 пинов на банк)
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DtBlobAtom {
    pub dlen: u32,           // Длина blob
    // Следом идут данные blob (dlen байт)
}

pub struct Eeprom {
    pub header: EepromHeader,
    pub vendor_info: VendorInfoAtom,
    pub gpio_map_bank0: GpioMapAtom,
    pub dt_blob: Option<Vec<u8>>, // DT blob может быть переменной длины
    pub gpio_map_bank1: Option<GpioMapAtom>, // Опционально
    // Можно добавить вектор для других атомов, если потребуется
}

// Можно добавить функции для парсинга и работы с этими структурами
impl Eeprom {
    /// Чтение структуры EEPROM из среза байт
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        use core::mem::size_of;
        use core::ptr::read_unaligned;

        if data.len() < size_of::<EepromHeader>() {
            return Err("Недостаточно данных для заголовка EEPROM");
        }
        // Читаем заголовок
        let header = unsafe { read_unaligned(data.as_ptr() as *const EepromHeader) };
        if &header.signature != b"R-Pi" {
            return Err("Неверная сигнатура EEPROM");
        }
        let mut offset = size_of::<EepromHeader>();
        let mut vendor_info = None;
        let mut gpio_map_bank0 = None;
        let mut dt_blob = None;
        let mut gpio_map_bank1 = None;
        for _ in 0..header.numatoms {
            if data.len() < offset + size_of::<AtomHeader>() {
                return Err("Недостаточно данных для AtomHeader");
            }
            let atom_header = unsafe { read_unaligned(data[offset..].as_ptr() as *const AtomHeader) };
            offset += size_of::<AtomHeader>();
            if data.len() < offset + atom_header.dlen as usize {
                return Err("Недостаточно данных для атома");
            }
            match AtomType::from(atom_header.atom_type) {
                AtomType::VendorInfo => {
                    if atom_header.dlen as usize >= size_of::<VendorInfoAtom>() {
                        vendor_info = Some(unsafe { read_unaligned(data[offset..].as_ptr() as *const VendorInfoAtom) });
                    }
                }
                AtomType::GpioMapBank0 => {
                    if atom_header.dlen as usize >= size_of::<GpioMapAtom>() {
                        gpio_map_bank0 = Some(unsafe { read_unaligned(data[offset..].as_ptr() as *const GpioMapAtom) });
                    }
                }
                AtomType::DtBlob => {
                    let dlen = atom_header.dlen as usize;
                    if dlen > 0 && data.len() >= offset + dlen {
                        dt_blob = Some(data[offset..offset + dlen].to_vec());
                    }
                }
                AtomType::GpioMapBank1 => {
                    if atom_header.dlen as usize >= size_of::<GpioMapAtom>() {
                        gpio_map_bank1 = Some(unsafe { read_unaligned(data[offset..].as_ptr() as *const GpioMapAtom) });
                    }
                }
                AtomType::Unknown => {
                    // Неизвестный тип атома — пропускаем
                }
            }
            offset += atom_header.dlen as usize;
        }
        Ok(Eeprom {
            header,
            vendor_info: vendor_info.ok_or("VendorInfo атом не найден")?,
            gpio_map_bank0: gpio_map_bank0.ok_or("GpioMapBank0 атом не найден")?,
            dt_blob,
            gpio_map_bank1,
        })
    }

    /// Проверяет, содержит ли EEPROM валидные данные (по сигнатуре и версии)
    pub fn is_valid(&self) -> bool {
        self.header.signature == *b"R-Pi" && self.header.version != 0
    }

    pub fn add_vendor_info(&mut self, atom: VendorInfoAtom) {
        self.vendor_info = atom;
        self.update_header();
    }
    pub fn add_gpio_map_bank0(&mut self, atom: GpioMapAtom) {
        self.gpio_map_bank0 = atom;
        self.update_header();
    }
    pub fn add_dt_blob(&mut self, blob: Vec<u8>) {
        self.dt_blob = Some(blob);
        self.update_header();
    }
    pub fn add_gpio_map_bank1(&mut self, atom: GpioMapAtom) {
        self.gpio_map_bank1 = Some(atom);
        self.update_header();
    }
    /// Пересчитать numatoms и eeplen после добавления атомов
    pub fn update_header(&mut self) {
        let mut numatoms = 2; // VendorInfo и GPIO bank0 всегда есть
        let mut eeplen = core::mem::size_of::<EepromHeader>()
            + core::mem::size_of::<AtomHeader>() * 2
            + core::mem::size_of::<VendorInfoAtom>()
            + core::mem::size_of::<GpioMapAtom>();
        if self.dt_blob.is_some() {
            numatoms += 1;
            if let Some(ref blob) = self.dt_blob {
                eeplen += core::mem::size_of::<AtomHeader>() + blob.len();
            }
        }
        if self.gpio_map_bank1.is_some() {
            numatoms += 1;
            eeplen += core::mem::size_of::<AtomHeader>() + core::mem::size_of::<GpioMapAtom>();
        }
        self.header.numatoms = numatoms;
        self.header.eeplen = eeplen as u32;
    }
}

impl From<u8> for AtomType {
    fn from(val: u8) -> Self {
        match val {
            0x01 => AtomType::VendorInfo,
            0x02 => AtomType::GpioMapBank0,
            0x03 => AtomType::DtBlob,
            0x04 => AtomType::GpioMapBank1,
            _ => AtomType::Unknown,
        }
    }
}

impl fmt::Debug for VendorInfoAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vendor_str = String::from_utf8_lossy(&self.vendor)
            .trim_end_matches('\0')
            .to_string();
        let product_str = String::from_utf8_lossy(&self.product)
            .trim_end_matches('\0')
            .to_string();
        write!(
            f,
            "VendorInfoAtom {{ vendor_id: {}, product_id: {}, product_ver: {}, vendor: \"{}\", product: \"{}\", uuid: {:?} }}",
            self.vendor_id, self.product_id, self.product_ver, vendor_str, product_str, self.uuid
        )
    }
}

pub fn write_to_eeprom_i2c(
    data: &[u8],
    dev_path: &str,
    addr: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut dev = LinuxI2CDevice::new(dev_path, addr)?;
    // EEPROM HAT: use page write (16 bytes per page) with 2-byte offset
    let page_size = 16;
    let mut offset = 0u16;
    while (offset as usize) < data.len() {
        let end = (offset as usize + page_size).min(data.len());
        let mut buf = Vec::with_capacity(2 + page_size);
        buf.push((offset >> 8) as u8);
        buf.push((offset & 0xFF) as u8);
        buf.extend_from_slice(&data[offset as usize..end]);
        dev.write(&buf)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        offset += (end - offset as usize) as u16;
    }
    Ok(())
}

pub fn read_from_eeprom_i2c(
    buf: &mut [u8],
    dev_path: &str,
    addr: u16,
    offset: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut dev = LinuxI2CDevice::new(dev_path, addr)?;
    // Сначала отправляем 2 байта offset (старший, младший)
    let offset_bytes = [(offset >> 8) as u8, (offset & 0xFF) as u8];
    dev.write(&offset_bytes)?;
    dev.read(buf)?;
    Ok(())
}