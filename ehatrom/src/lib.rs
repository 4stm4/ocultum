//  _  _       _             _  _
// | || |  ___| |_ _ __ ___ | || |
// | || |_/ __| __| '_ ` _ \| || |_
// |__   _\__ | |_| | | | | |__   _|
//   |_| |___/\__|_|_|_| |_|  |_|
//  ___   ___ _   _| | |_ _   _ _ __ ___
// / _ \ / __| | | | | __| | | | '_ ` _ \
//| (_) | (__| |_| | | |_| |_| | | | | | |
// \___/ \___|\__,_|_|\__|\__,_|_| |_| |_|
//! # ehatrom — EEPROM HAT library for Raspberry Pi HATs
//!
//! EEPROM (de)serialization, I2C read/write (Linux), CRC32, custom atoms, CLI example.
//!
//! - [Documentation (docs.rs)](https://docs.rs/ehatrom)
//! - [GitHub](https://github.com/youruser/ehatrom)
//!
//! ## Example
//!
//! ```rust
//! use ehatrom::{Eeprom, VendorInfoAtom, GpioMapAtom};
//!
//! let vendor_info = VendorInfoAtom::new(
//!     0x1234, 0x5678, 1, "MyVendor", "MyHAT", [0u8; 16]
//! );
//! let gpio = GpioMapAtom { flags: 0, pins: [0; 28] };
//! let mut eeprom = Eeprom {
//!     header: Default::default(),
//!     vendor_info,
//!     gpio_map_bank0: gpio,
//!     dt_blob: None,
//!     gpio_map_bank1: None,
//!     custom_atoms: vec![],
//! };
//! eeprom.update_header();
//! ```
//!

use core::fmt;
use crc32fast::Hasher;
use i2cdev::core::I2CDevice;
#[cfg(target_os = "linux")]
use i2cdev::linux::LinuxI2CDevice;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EepromHeader {
    pub signature: [u8; 4], // Always 0x52 0x2D 0x50 0x69 ("R-Pi")
    pub version: u8,        // Format version (0x01 for first version)
    pub reserved: u8,       // 0x00
    pub numatoms: u16,      // Количество атомов (Little Endian)
    pub eeplen: u32,        // Общая длина данных EEPROM (LE)
}

impl Default for EepromHeader {
    fn default() -> Self {
        EepromHeader {
            signature: *b"R-Pi",
            version: 1,
            reserved: 0,
            numatoms: 0,
            eeplen: 0,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct AtomHeader {
    pub atom_type: u8, // Atom type
    pub count: u8,     // Number of structures in atom (usually 1)
    pub dlen: u16,     // Data length (LE)
    pub reserved: u32, // Reserved (0)
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
#[derive(Clone, Copy)]
pub struct VendorInfoAtom {
    pub vendor_id: u16,    // Vendor ID
    pub product_id: u16,   // Product ID
    pub product_ver: u16,  // Product version
    pub vendor: [u8; 16],  // Vendor name (null-terminated)
    pub product: [u8; 16], // Product name (null-terminated)
    pub uuid: [u8; 16],    // UUID
}

impl fmt::Debug for VendorInfoAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vendor_str = String::from_utf8_lossy(&self.vendor)
            .trim_end_matches('\0')
            .to_string();
        let product_str = String::from_utf8_lossy(&self.product)
            .trim_end_matches('\0')
            .to_string();
        let vendor_id = self.vendor_id;
        let product_id = self.product_id;
        let product_ver = self.product_ver;
        let uuid = self.uuid;
        write!(
            f,
            "VendorInfoAtom {{ vendor_id: {vendor_id}, product_id: {product_id}, product_ver: {product_ver}, vendor: \"{vendor_str}\", product: \"{product_str}\", uuid: {uuid:?} }}"
        )
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GpioMapAtom {
    pub flags: u16,     // Флаги GPIO
    pub pins: [u8; 28], // Карта пинов (28 пинов на банк)
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DtBlobAtom {
    pub dlen: u32, // Длина blob
                   // Следом идут данные blob (dlen байт)
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct CustomAtom<const N: usize> {
    pub atom_type: u8, // Пользовательский тип атома (>= 0x80)
    pub data: [u8; N], // Пользовательские данные
}

impl<const N: usize> fmt::Debug for CustomAtom<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Безопасно копируем packed-поле data
        let self_ptr = self as *const Self as *const u8;
        let data_offset = core::mem::size_of::<u8>();
        let data_ptr = unsafe { self_ptr.add(data_offset) };
        let mut data = [0u8; N];
        unsafe {
            core::ptr::copy_nonoverlapping(data_ptr, data.as_mut_ptr(), N);
        }
        write!(
            f,
            "CustomAtom {{ atom_type: 0x{:02X}, data: {:?} }}",
            self.atom_type,
            &data[..]
        )
    }
}

pub enum EepromAtom {
    VendorInfo(VendorInfoAtom),
    GpioMapBank0(GpioMapAtom),
    DtBlob(Vec<u8>),
    GpioMapBank1(GpioMapAtom),
    Custom(Vec<u8>, u8), // (данные, тип)
}

pub struct Eeprom {
    pub header: EepromHeader,
    pub vendor_info: VendorInfoAtom,
    pub gpio_map_bank0: GpioMapAtom,
    pub dt_blob: Option<Vec<u8>>, // DT blob can be variable length
    pub gpio_map_bank1: Option<GpioMapAtom>, // Optional
    pub custom_atoms: Vec<(u8, Vec<u8>)>, // (atom_type, data)
}

// Можно добавить функции для парсинга и работы с этими структурами
impl Eeprom {
    /// Reads EEPROM structure from a byte slice
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        use core::mem::size_of;
        use core::ptr::read_unaligned;

        if data.len() < size_of::<EepromHeader>() {
            return Err("Not enough data for EEPROM header");
        }
        // Read header
        let header = unsafe { read_unaligned(data.as_ptr() as *const EepromHeader) };
        if &header.signature != b"R-Pi" {
            return Err("Invalid EEPROM signature");
        }
        let mut offset = size_of::<EepromHeader>();
        let mut vendor_info = None;
        let mut gpio_map_bank0 = None;
        let mut dt_blob = None;
        let mut gpio_map_bank1 = None;
        let mut custom_atoms = Vec::new();
        for _ in 0..header.numatoms {
            if data.len() < offset + size_of::<AtomHeader>() {
                return Err("Not enough data for AtomHeader");
            }
            let atom_header =
                unsafe { read_unaligned(data[offset..].as_ptr() as *const AtomHeader) };
            offset += size_of::<AtomHeader>();
            if data.len() < offset + atom_header.dlen as usize {
                return Err("Not enough data for atom");
            }
            match AtomType::from(atom_header.atom_type) {
                AtomType::VendorInfo => {
                    if atom_header.dlen as usize >= size_of::<VendorInfoAtom>() {
                        vendor_info = Some(unsafe {
                            read_unaligned(data[offset..].as_ptr() as *const VendorInfoAtom)
                        });
                    }
                }
                AtomType::GpioMapBank0 => {
                    if atom_header.dlen as usize >= size_of::<GpioMapAtom>() {
                        gpio_map_bank0 = Some(unsafe {
                            read_unaligned(data[offset..].as_ptr() as *const GpioMapAtom)
                        });
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
                        gpio_map_bank1 = Some(unsafe {
                            read_unaligned(data[offset..].as_ptr() as *const GpioMapAtom)
                        });
                    }
                }
                AtomType::Unknown => {
                    // Save custom atom (type and data)
                    let dlen = atom_header.dlen as usize;
                    if dlen > 0 && data.len() >= offset + dlen {
                        custom_atoms
                            .push((atom_header.atom_type, data[offset..offset + dlen].to_vec()));
                    }
                }
            }
            offset += atom_header.dlen as usize;
        }
        Ok(Eeprom {
            header,
            vendor_info: vendor_info.ok_or("VendorInfo atom not found")?,
            gpio_map_bank0: gpio_map_bank0.ok_or("GpioMapBank0 atom not found")?,
            dt_blob,
            gpio_map_bank1,
            custom_atoms,
        })
    }

    /// Checks if EEPROM contains valid data (by signature and version)
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
    pub fn add_custom_atom(&mut self, atom_type: u8, data: Vec<u8>) {
        self.custom_atoms.push((atom_type, data));
        self.update_header();
    }
    /// Recalculate numatoms and eeplen after adding atoms
    pub fn update_header(&mut self) {
        let mut numatoms = 2; // VendorInfo and GPIO bank0 are always present
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
        for (_atom_type, data) in &self.custom_atoms {
            numatoms += 1;
            eeplen += core::mem::size_of::<AtomHeader>() + data.len();
        }
        self.header.numatoms = numatoms;
        self.header.eeplen = eeplen as u32;
    }

    /// Serialize with CRC32 appended (4 bytes LE)
    pub fn serialize_with_crc(&self) -> Vec<u8> {
        let mut data = self.serialize();
        let mut hasher = Hasher::new();
        hasher.update(&data);
        let crc = hasher.finalize();
        data.extend_from_slice(&crc.to_le_bytes());
        data
    }
    /// CRC32 check (expects last 4 bytes to be CRC32 LE)
    pub fn verify_crc(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }
        let (content, crc_bytes) = data.split_at(data.len() - 4);
        let mut hasher = Hasher::new();
        hasher.update(content);
        let crc = hasher.finalize();
        crc_bytes == crc.to_le_bytes()
    }

    /// Serialize EEPROM structure to Vec<u8> (without CRC)
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // EEPROM header
        let header_ptr = &self.header as *const _ as *const u8;
        bytes.extend_from_slice(unsafe {
            std::slice::from_raw_parts(header_ptr, core::mem::size_of::<EepromHeader>())
        });
        // VendorInfo
        let atom_header = AtomHeader {
            atom_type: AtomType::VendorInfo as u8,
            count: 1,
            dlen: core::mem::size_of::<VendorInfoAtom>() as u16,
            reserved: 0,
        };
        let atom_ptr = &atom_header as *const _ as *const u8;
        bytes.extend_from_slice(unsafe {
            std::slice::from_raw_parts(atom_ptr, core::mem::size_of::<AtomHeader>())
        });
        let vendor_ptr = &self.vendor_info as *const _ as *const u8;
        bytes.extend_from_slice(unsafe {
            std::slice::from_raw_parts(vendor_ptr, core::mem::size_of::<VendorInfoAtom>())
        });
        // GPIO bank0
        let atom_header = AtomHeader {
            atom_type: AtomType::GpioMapBank0 as u8,
            count: 1,
            dlen: core::mem::size_of::<GpioMapAtom>() as u16,
            reserved: 0,
        };
        let atom_ptr = &atom_header as *const _ as *const u8;
        bytes.extend_from_slice(unsafe {
            std::slice::from_raw_parts(atom_ptr, core::mem::size_of::<AtomHeader>())
        });
        let gpio_ptr = &self.gpio_map_bank0 as *const _ as *const u8;
        bytes.extend_from_slice(unsafe {
            std::slice::from_raw_parts(gpio_ptr, core::mem::size_of::<GpioMapAtom>())
        });
        // DT blob
        if let Some(ref blob) = self.dt_blob {
            let atom_header = AtomHeader {
                atom_type: AtomType::DtBlob as u8,
                count: 1,
                dlen: blob.len() as u16,
                reserved: 0,
            };
            let atom_ptr = &atom_header as *const _ as *const u8;
            bytes.extend_from_slice(unsafe {
                std::slice::from_raw_parts(atom_ptr, core::mem::size_of::<AtomHeader>())
            });
            bytes.extend_from_slice(blob);
        }
        // GPIO bank1
        if let Some(ref bank1) = self.gpio_map_bank1 {
            let atom_header = AtomHeader {
                atom_type: AtomType::GpioMapBank1 as u8,
                count: 1,
                dlen: core::mem::size_of::<GpioMapAtom>() as u16,
                reserved: 0,
            };
            let atom_ptr = &atom_header as *const _ as *const u8;
            bytes.extend_from_slice(unsafe {
                std::slice::from_raw_parts(atom_ptr, core::mem::size_of::<AtomHeader>())
            });
            let gpio_ptr = bank1 as *const _ as *const u8;
            bytes.extend_from_slice(unsafe {
                std::slice::from_raw_parts(gpio_ptr, core::mem::size_of::<GpioMapAtom>())
            });
        }
        // Custom atoms
        for (atom_type, data) in &self.custom_atoms {
            let atom_header = AtomHeader {
                atom_type: *atom_type,
                count: 1,
                dlen: data.len() as u16,
                reserved: 0,
            };
            let atom_ptr = &atom_header as *const _ as *const u8;
            bytes.extend_from_slice(unsafe {
                std::slice::from_raw_parts(atom_ptr, core::mem::size_of::<AtomHeader>())
            });
            bytes.extend_from_slice(data);
        }
        bytes
    }

    pub fn set_version(&mut self, version: u8) {
        self.header.version = version;
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

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
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

impl VendorInfoAtom {
    /// Creates VendorInfoAtom from strings (automatically trims/pads with zeros)
    pub fn new(
        vendor_id: u16,
        product_id: u16,
        product_ver: u16,
        vendor: &str,
        product: &str,
        uuid: [u8; 16],
    ) -> Self {
        let mut vendor_arr = [0u8; 16];
        let mut product_arr = [0u8; 16];
        let vendor_bytes = vendor.as_bytes();
        let product_bytes = product.as_bytes();
        let vendor_len = vendor_bytes.len().min(15); // leave space for null-terminator
        let product_len = product_bytes.len().min(15);
        vendor_arr[..vendor_len].copy_from_slice(&vendor_bytes[..vendor_len]);
        product_arr[..product_len].copy_from_slice(&product_bytes[..product_len]);
        // null-terminator is already present, as arrays are zero-filled
        VendorInfoAtom {
            vendor_id,
            product_id,
            product_ver,
            vendor: vendor_arr,
            product: product_arr,
            uuid,
        }
    }
}
