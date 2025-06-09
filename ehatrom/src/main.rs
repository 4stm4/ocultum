//  _  _       _             _  _
// | || |  ___| |_ _ __ ___ | || |
// | || |_/ __| __| '_ ` _ \| || |_
// |__   _\__ | |_| | | | | |__   _|
//   |_| |___/\__|_|_|_| |_|  |_|
//  ___   ___ _   _| | |_ _   _ _ __ ___
// / _ \ / __| | | | | __| | | | '_ ` _ \
//| (_) | (__| |_| | | |_| |_| | | | | | |
// \___/ \___|\__,_|_|\__|\__,_|_| |_| |_|

use ehatrom::{Eeprom, GpioMapAtom, VendorInfoAtom};

fn main() {
    // --- Build structure for writing ---
    let mut eeprom = Eeprom {
        header: Default::default(),
        vendor_info: VendorInfoAtom {
            vendor_id: 0x1234,
            product_id: 0x5678,
            product_ver: 1, // product version in development — usually 1
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
        gpio_map_bank0: GpioMapAtom {
            flags: 0,
            pins: [0; 28],
        },
        dt_blob: None,
        gpio_map_bank1: None,
        custom_atoms: Vec::new(),
    };
    eeprom.update_header();
    // Добавление пользовательского атома (например, тип 0x80, данные "hello world")
    eeprom.add_custom_atom(0x80, b"hello world".to_vec());
    // --- Serialization ---
    #[cfg(target_os = "linux")]
    {
        // Serialization with CRC32
        let bytes_with_crc = eeprom.serialize_with_crc();
        let dev_path = "/dev/i2c-0";
        let addr = 0x50;
        match ehatrom::write_to_eeprom_i2c(&bytes_with_crc, dev_path, addr) {
            Ok(_) => println!("Data successfully written to EEPROM!"),
            Err(e) => {
                eprintln!("Error writing to EEPROM: {e}");
                return;
            }
        }
        // EEPROM may require a delay after writing
        std::thread::sleep(std::time::Duration::from_millis(10));
        // --- Read and check ---
        let len = bytes_with_crc.len();
        let mut data = vec![0u8; len];
        match ehatrom::read_from_eeprom_i2c(&mut data, dev_path, addr, 0x0000) {
            Ok(_) => {
                // For debugging: print first 16 bytes in hex
                print!("EEPROM HEX: ");
                for b in data.iter().take(16) {
                    print!("{b:02X} ");
                }
            }
            Err(e) => {
                eprintln!("Error reading from I2C: {e}");
                return;
            }
        }
        match Eeprom::from_bytes(&data[..data.len() - 4]) {
            Ok(eeprom) => {
                if eeprom.is_valid() {
                    println!("EEPROM header: {:?}", eeprom.header);
                    println!("Vendor info: {:?}", eeprom.vendor_info);
                    println!("GPIO map bank0: {:?}", eeprom.gpio_map_bank0);
                    if !eeprom.custom_atoms.is_empty() {
                        println!("Custom atoms:");
                        for (atom_type, data) in &eeprom.custom_atoms {
                            print!("  Type 0x{atom_type:02X}: ");
                            for b in data {
                                print!("{b:02X} ");
                            }
                            if let Ok(s) = std::str::from_utf8(data) {
                                print!(" (as string: \"{s}\")");
                            }
                            println!();
                        }
                    }
                } else {
                    println!("EEPROM is empty or uninitialized (invalid signature/version)");
                }
            }
            Err(e) => {
                eprintln!("EEPROM parsing error: {e}");
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        println!("I2C EEPROM read/write is only available on Linux.");
    }
}
