/*
  4STM4
  ocultum
*/
use ehatrom::*;

fn make_vendor() -> VendorInfoAtom {
    VendorInfoAtom::new(0x1234, 0x5678, 1, "testvendor", "testproduct", [1; 16])
}
fn make_gpio() -> GpioMapAtom {
    GpioMapAtom {
        flags: 0xAA55,
        pins: [1; 28],
    }
}

#[test]
fn test_eeprom_validity() {
    let mut eeprom = Eeprom {
        header: Default::default(),
        vendor_info: make_vendor(),
        gpio_map_bank0: make_gpio(),
        dt_blob: None,
        gpio_map_bank1: None,
        custom_atoms: vec![],
    };
    assert!(eeprom.is_valid());
    eeprom.header.signature = [0, 0, 0, 0];
    assert!(!eeprom.is_valid());
}

#[test]
fn test_set_version() {
    let mut eeprom = Eeprom {
        header: Default::default(),
        vendor_info: make_vendor(),
        gpio_map_bank0: make_gpio(),
        dt_blob: None,
        gpio_map_bank1: None,
        custom_atoms: vec![],
    };
    eeprom.set_version(42);
    assert_eq!(eeprom.header.version, 42);
}

#[test]
fn test_add_atoms_and_update_header() {
    let mut eeprom = Eeprom {
        header: Default::default(),
        vendor_info: make_vendor(),
        gpio_map_bank0: make_gpio(),
        dt_blob: None,
        gpio_map_bank1: None,
        custom_atoms: vec![],
    };
    let orig_atoms = eeprom.header.numatoms;
    eeprom.add_gpio_map_bank1(make_gpio());
    assert!(eeprom.gpio_map_bank1.is_some());
    assert!(eeprom.header.numatoms > orig_atoms);
    eeprom.add_dt_blob(vec![1, 2, 3, 4]);
    assert!(eeprom.dt_blob.is_some());
    let orig_atoms = eeprom.header.numatoms;
    eeprom.add_custom_atom(0x80, b"custom".to_vec());
    assert_eq!(eeprom.custom_atoms.len(), 1);
    assert!(eeprom.header.numatoms > orig_atoms);
}

#[test]
fn test_serialize_and_deserialize() {
    let mut eeprom = Eeprom {
        header: Default::default(),
        vendor_info: make_vendor(),
        gpio_map_bank0: make_gpio(),
        dt_blob: Some(vec![1, 2, 3]),
        gpio_map_bank1: Some(make_gpio()),
        custom_atoms: vec![(0x80, b"custom".to_vec())],
    };
    eeprom.update_header();
    let bytes = eeprom.serialize_with_crc();
    assert!(Eeprom::verify_crc(&bytes));
    let without_crc = &bytes[..bytes.len() - 4];
    let parsed = Eeprom::from_bytes(without_crc).unwrap();
    let vendor_id = parsed.vendor_info.vendor_id;
    let flags = parsed.gpio_map_bank0.flags;
    assert_eq!(vendor_id, 0x1234);
    assert_eq!(flags, 0xAA55);
    assert!(parsed.dt_blob.is_some());
    assert!(parsed.gpio_map_bank1.is_some());
    assert_eq!(parsed.custom_atoms.len(), 1);
}

#[test]
fn test_crc_check() {
    let mut eeprom = Eeprom {
        header: Default::default(),
        vendor_info: make_vendor(),
        gpio_map_bank0: make_gpio(),
        dt_blob: None,
        gpio_map_bank1: None,
        custom_atoms: vec![],
    };
    let mut bytes = eeprom.serialize_with_crc();
    assert!(Eeprom::verify_crc(&bytes));
    bytes[10] ^= 0xFF; // corrupt
    assert!(!Eeprom::verify_crc(&bytes));
}

#[test]
fn test_from_bytes_invalid() {
    // Недостаточно данных
    assert!(Eeprom::from_bytes(&[1, 2, 3]).is_err());
    // Неверная сигнатура
    let mut eeprom = Eeprom {
        header: Default::default(),
        vendor_info: make_vendor(),
        gpio_map_bank0: make_gpio(),
        dt_blob: None,
        gpio_map_bank1: None,
        custom_atoms: vec![],
    };
    eeprom.header.signature = [0, 0, 0, 0];
    let bytes = eeprom.serialize();
    assert!(Eeprom::from_bytes(&bytes).is_err());
}
