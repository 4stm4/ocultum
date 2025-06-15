// Условная компиляция для Linux
#[cfg(feature = "linux")]
use linux_embedded_hal::{Delay, I2cdev}; 

#[cfg(feature = "linux")] // Оборачиваем импорты, используемые в init_oled
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10}, // Порядок изменен согласно fmt
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
#[cfg(feature = "linux")] // Оборачиваем импорты, используемые в init_oled
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    loop {}
}

#[cfg(not(target_os = "none"))]
fn main() {
    eprintln!("P1: Host mode initialized (stderr)");

    eprintln!("P2: Before linux feature check (stderr)");

    #[cfg(feature = "linux")]
    {
        eprintln!("P3: Linux feature IS enabled. Attempting real I2C. (stderr)");
        match I2cdev::new("/dev/i2c-1") {
            // Оставляем /dev/i2c-1, измените при необходимости
            Ok(i2c) => {
                eprintln!("P3.1: I2C device /dev/i2c-1 opened successfully. (stderr)");
                let delay = Delay;
                init_oled(i2c, delay);
            }
            Err(e) => {
                eprintln!(
                    "P3.2: ERROR - Failed to open I2C device /dev/i2c-1: {:?} (stderr)",
                    e
                );
                eprintln!("P3.3: Please check the I2C device path and permissions. (stderr)");
            }
        }
    }

    #[cfg(not(feature = "linux"))]
    {
        eprintln!("P4: Linux feature IS NOT enabled. OLED initialization skipped. (stderr)");
    }

    eprintln!("P5: After feature checks. Main is ending. (stderr)");
}

#[cfg(feature = "linux")] // Оборачиваем всю функцию
fn init_oled<I2C, D>(i2c: I2C, mut delay: D)
where
    I2C: embedded_hal::i2c::I2c,
    I2C::Error: core::fmt::Debug,
    D: embedded_hal::delay::DelayNs,
{
    eprintln!("INIT_OLED: Attempting to initialize display... (stderr)");
    let interface = I2CDisplayInterface::new(i2c);
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    if disp.init().is_err() {
        eprintln!("INIT_OLED: ERROR - Failed to initialize OLED display. (stderr)");
        return;
    }
    if disp.clear(BinaryColor::Off).is_err() {
        eprintln!("INIT_OLED: ERROR - Failed to clear OLED display. (stderr)");
        return;
    }

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    if Text::with_baseline("Hello, Rust!", Point::new(0, 10), text_style, Baseline::Top)
        .draw(&mut disp)
        .is_err()
    {
        eprintln!("INIT_OLED: ERROR - Failed to draw text on OLED. (stderr)");
        return;
    }

    if disp.flush().is_err() {
        eprintln!("INIT_OLED: ERROR - Failed to flush OLED display buffer. (stderr)");
        return;
    }

    eprintln!("INIT_OLED: OLED display initialized and text should be visible. (stderr)");

    #[cfg(not(target_os = "none"))]
    delay.delay_ms(5000);
}
