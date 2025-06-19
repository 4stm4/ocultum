use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

pub fn display_eeprom_info<I2C, D>(i2c: I2C, _delay: D, _address: u8, eeprom: &ehatrom::Eeprom)
where
    I2C: embedded_hal::i2c::I2c,
    I2C::Error: core::fmt::Debug,
    D: embedded_hal::delay::DelayNs,
{
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    if disp.init().is_err() {
        eprintln!("ERROR: Failed to initialize OLED display");
        return;
    }
    if disp.clear(BinaryColor::Off).is_err() {
        eprintln!("ERROR: Failed to clear OLED display");
        return;
    }

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let vendor = String::from_utf8_lossy(&eeprom.vendor_info.vendor)
        .trim_end_matches(char::from(0))
        .to_string();
    let product = String::from_utf8_lossy(&eeprom.vendor_info.product)
        .trim_end_matches(char::from(0))
        .to_string();

    let _ = Text::with_baseline(
        &format!("Vendor: {vendor}"),
        Point::new(0, 8),
        text_style,
        Baseline::Top,
    )
    .draw(&mut disp);
    let _ = Text::with_baseline(
        &format!("Product: {product}"),
        Point::new(0, 20),
        text_style,
        Baseline::Top,
    )
    .draw(&mut disp);

    let _ = disp.flush();
    eprintln!("EEPROM info displayed on OLED");
}
