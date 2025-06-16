use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

pub fn init_oled<I2C, D>(i2c: I2C, mut delay: D, address: u8)
where
    I2C: embedded_hal::i2c::I2c,
    I2C::Error: core::fmt::Debug,
    D: embedded_hal::delay::DelayNs,
{
    eprintln!("Initializing OLED display...");
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

    if Text::with_baseline("Ocultum OLED", Point::new(0, 8), text_style, Baseline::Top)
        .draw(&mut disp)
        .is_err()
        || Text::with_baseline("Display Test", Point::new(0, 22), text_style, Baseline::Top)
            .draw(&mut disp)
            .is_err()
        || Text::with_baseline(
            &format!("I2C: {address}"),
            Point::new(0, 36),
            text_style,
            Baseline::Top,
        )
        .draw(&mut disp)
        .is_err()
        || Text::with_baseline("Running OK!", Point::new(0, 50), text_style, Baseline::Top)
            .draw(&mut disp)
            .is_err()
    {
        eprintln!("ERROR: Failed to display text on OLED display");
        return;
    }

    if disp.flush().is_err() {
        eprintln!("ERROR: Failed to update OLED display buffer");
        return;
    }

    eprintln!("OLED display successfully initialized, text should be visible");

    delay.delay_ms(5000);
}
