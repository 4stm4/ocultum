//! Модуль ssd1306 предназначен только для Linux.

use ehatrom::Eeprom;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::i2c::I2c;
use linux_embedded_hal::I2cdev;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

static mut LAST_EEPROM: Option<Eeprom> = None;

pub fn set_eeprom(eeprom: Eeprom) {
    unsafe {
        LAST_EEPROM = Some(eeprom);
    }
}

pub fn display_eeprom_info() {
    let (bus, _address) = match detect_display(20) {
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
    let eeprom_val = unsafe { core::ptr::addr_of!(LAST_EEPROM).read() };
    let eeprom = match &eeprom_val {
        Some(e) => e,
        None => {
            eprintln!("Нет данных EEPROM для отображения!");
            return;
        }
    };
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let vendor = clean_string(&eeprom.vendor_info.vendor);
    let product = clean_string(&eeprom.vendor_info.product);

    let _ = Text::with_baseline(
        &format!("Vendor: {vendor}"),
        Point::new(0, 8),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display);
    let _ = Text::with_baseline(
        &format!("Product: {product}"),
        Point::new(0, 20),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display);
    display.flush().unwrap();
}

fn detect_display(max_bus: u8) -> Option<(u8, u8)> {
    (0..=max_bus).find_map(|bus| {
        let i2c_path = format!("/dev/i2c-{bus}");
        let Ok(mut i2c) = I2cdev::new(&i2c_path) else {
            return None;
        };
        [0x3C, 0x3D]
            .iter()
            .find(|&&addr| {
                let mut buf = [0u8; 1];
                let mut ops = [embedded_hal::i2c::Operation::Read(&mut buf)];
                i2c.transaction(addr, &mut ops).is_ok()
            })
            .map(|&addr| (bus, addr))
    })
}

fn clean_string(bytes: &[u8]) -> &str {
    let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..null_pos]).unwrap_or("")
}
